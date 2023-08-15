use crate::error::LavalinkResult;
use crate::model::*;
use crate::node;
use crate::player_context::*;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use arc_swap::ArcSwap;
use dashmap::DashMap;
use reqwest::{header::HeaderMap, Client as ReqwestClient};

#[derive(Debug, Clone)]
pub struct LavalinkClient {
    pub nodes: Arc<Vec<node::Node>>,
    pub players: Arc<DashMap<GuildId, PlayerContext>>,
    pub events: events::Events,
    //user_data: Arc<RwLock<TypeMap>>
}

impl LavalinkClient {
    pub fn new(events: events::Events, hostnames: Vec<node::NodeBuilder>) -> LavalinkClient {
        let mut nodes = Vec::new();

        for (idx, i) in hostnames.into_iter().enumerate() {
            let mut headers = HeaderMap::new();
            headers.insert("Authorization", i.password.parse().unwrap());
            headers.insert("User-Id", i.user_id.0.to_string().parse().unwrap());

            if let Some(session_id) = &i.session_id {
                headers.insert("Session-Id", session_id.parse().unwrap());
            }

            headers.insert(
                "Client-Name",
                format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                    .to_string()
                    .parse()
                    .unwrap(),
            );

            let rest_client = ReqwestClient::builder()
                .default_headers(headers)
                .build()
                .unwrap();

            let node = if i.is_ssl {
                let http = crate::http::Http {
                    rest_address: format!("https://{}/v4", i.hostname),
                    rest_address_versionless: format!("https://{}", i.hostname),
                    rest_client,
                };

                node::Node {
                    id: idx,
                    websocket_address: format!("wss://{}/v4/websocket", i.hostname),
                    http,
                    events: i.events,
                    password: i.password,
                    user_id: i.user_id,
                    is_running: AtomicBool::new(false),
                    session_id: ArcSwap::new(if let Some(id) = i.session_id {
                        id.into()
                    } else {
                        idx.to_string().into()
                    }),
                }
            } else {
                let http = crate::http::Http {
                    rest_address: format!("http://{}/v4", i.hostname),
                    rest_address_versionless: format!("http://{}", i.hostname),
                    rest_client,
                };

                node::Node {
                    id: idx,
                    websocket_address: format!("ws://{}/v4/websocket", i.hostname),
                    http,
                    events: i.events,
                    password: i.password,
                    user_id: i.user_id,
                    is_running: AtomicBool::new(false),
                    session_id: ArcSwap::new(if let Some(id) = i.session_id {
                        id.into()
                    } else {
                        idx.to_string().into()
                    }),
                }
            };

            nodes.push(node);
        }

        LavalinkClient {
            nodes: Arc::new(nodes),
            players: Arc::new(DashMap::new()),
            events,
        }
    }

    pub async fn start(&self) {
        for node in &*self.nodes {
            if let Err(why) = node.connect(self.clone()).await {
                error!("Failed to connect to the lavalink websocket: {}", why);
            }
        }

        let lavalink_client = self.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;

                for node in &*lavalink_client.nodes {
                    if !node.is_running.load(Ordering::Relaxed) {
                        if let Err(why) = node.connect(lavalink_client.clone()).await {
                            error!("Failed to connect to the lavalink websocket: {}", why);
                        }
                    }
                }
            }
        });
    }

    pub fn get_node_for_guild(&self, guild_id: GuildId) -> &node::Node {
        self.nodes
            .get(guild_id.0 as usize % self.nodes.len())
            .unwrap()
    }

    pub fn get_player_context(&self, guild_id: GuildId) -> Option<PlayerContext> {
        self.players.get(&guild_id).map(|x| x.clone())
    }

    pub async fn create_player(
        &self,
        guild_id: GuildId,
        connection_info: &player::ConnectionInfo,
    ) -> LavalinkResult<PlayerContext> {
        let node = self.get_node_for_guild(guild_id);

        if let Some(x) = self.players.get(&guild_id) {
            return Ok(x.clone());
        }

        let player = node
            .http
            .update_player(
                guild_id,
                &node.session_id.load(),
                &http::UpdatePlayer {
                    voice: Some(connection_info.clone()),
                    ..Default::default()
                },
                true,
            )
            .await?;

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let player_dummy = PlayerContext {
            guild_id,
            client: self.clone(),
            tx,
        };

        let player_context = PlayerContextInner {
            guild_id,
            queue: VecDeque::new(),
            player_data: player,
            dummy: player_dummy.clone(),
            last_should_continue: true,
        };

        player_context.start(rx).await;

        self.players.insert(guild_id, player_dummy.clone());

        Ok(player_dummy)
    }

    pub async fn delete_player(&self, guild_id: GuildId) -> LavalinkResult<()> {
        let node = self.get_node_for_guild(guild_id);

        node.http
            .delete_player(guild_id, &node.session_id.load())
            .await?;

        if let Some((_, player)) = self.players.remove(&guild_id) {
            player.close()?;
        }

        Ok(())
    }

    pub async fn load_tracks(&self, guild_id: GuildId, term: &str) -> LavalinkResult<track::Track> {
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.load_tracks(term).await?;

        Ok(result)
    }

    pub async fn decode_track(
        &self,
        guild_id: GuildId,
        track: &str,
    ) -> LavalinkResult<track::TrackData> {
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.decode_track(track).await?;

        Ok(result)
    }

    pub async fn decode_tracks(
        &self,
        guild_id: GuildId,
        tracks: &[String],
    ) -> LavalinkResult<Vec<track::TrackData>> {
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.decode_tracks(tracks).await?;

        Ok(result)
    }

    pub async fn request_version(&self, guild_id: GuildId) -> LavalinkResult<String> {
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.version().await?;

        Ok(result)
    }

    pub async fn request_stats(&self, guild_id: GuildId) -> LavalinkResult<events::Stats> {
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.stats().await?;

        Ok(result)
    }

    pub async fn request_info(&self, guild_id: GuildId) -> LavalinkResult<http::Info> {
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.info().await?;

        Ok(result)
    }

    pub async fn request_player(&self, guild_id: GuildId) -> LavalinkResult<player::Player> {
        let node = self.get_node_for_guild(guild_id);

        let result = node
            .http
            .get_player(guild_id, &node.session_id.load())
            .await?;

        Ok(result)
    }

    pub async fn request_all_players(
        &self,
        guild_id: GuildId,
    ) -> LavalinkResult<Vec<player::Player>> {
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.get_players(&node.session_id.load()).await?;

        Ok(result)
    }
}
