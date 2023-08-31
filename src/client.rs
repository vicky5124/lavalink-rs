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

#[derive(Clone)]
#[cfg_attr(not(feature = "user-data"), derive(Debug))]
/// The main client, where everything gets done, from events to requests to management.
pub struct LavalinkClient {
    pub nodes: Arc<Vec<node::Node>>,
    pub players: Arc<DashMap<GuildId, PlayerContext>>,
    pub events: events::Events,
    #[cfg(feature = "user-data")]
    pub user_data: Arc<parking_lot::RwLock<typemap_rev::TypeMap>>,
}

impl LavalinkClient {
    /// Create a new Lavalink Client.
    ///
    /// # Parameters
    ///
    /// - `events`: The lavalink event handler.
    /// - `nodes`: List of nodes to connect to.
    pub fn new(events: events::Events, nodes: Vec<node::NodeBuilder>) -> LavalinkClient {
        let mut built_nodes = Vec::new();

        for (idx, i) in nodes.into_iter().enumerate() {
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

            built_nodes.push(node);
        }

        LavalinkClient {
            nodes: Arc::new(built_nodes),
            players: Arc::new(DashMap::new()),
            events,
            #[cfg(feature = "user-data")]
            user_data: Arc::new(parking_lot::RwLock::new(typemap_rev::TypeMap::new())),
        }
    }

    /// Establish the connection(s) and start listening for events.
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

    /// Get the node assigned to a guild.
    pub fn get_node_for_guild(&self, guild_id: impl Into<GuildId>) -> &node::Node {
        let guild_id = guild_id.into();

        self.nodes
            .get(guild_id.0 as usize % self.nodes.len())
            .unwrap()
    }

    /// Get the player context for a guild, if it exists.
    pub fn get_player_context(&self, guild_id: impl Into<GuildId>) -> Option<PlayerContext> {
        let guild_id = guild_id.into();

        self.players.get(&guild_id).map(|x| x.clone())
    }

    /// Creates a new player without a context.
    ///
    /// Calling this method is required to play tracks on a guild.
    pub async fn create_player(
        &self,
        guild_id: impl Into<GuildId>,
        connection_info: impl Into<player::ConnectionInfo>,
    ) -> LavalinkResult<player::Player> {
        let guild_id = guild_id.into();
        let connection_info = connection_info.into();

        let node = self.get_node_for_guild(guild_id);

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

        Ok(player)
    }

    /// Creates a new player with context.
    ///
    /// Calling this method is required to create the initial player, and be able to use the built-in queue.
    pub async fn create_player_context(
        &self,
        guild_id: impl Into<GuildId>,
        connection_info: impl Into<player::ConnectionInfo>,
    ) -> LavalinkResult<PlayerContext> {
        let guild_id = guild_id.into();
        let connection_info = connection_info.into();

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
            #[cfg(feature = "user-data")]
            user_data: Arc::new(parking_lot::RwLock::new(typemap_rev::TypeMap::new())),
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

    /// Deletes and closes a specific player context, if it exists.
    pub async fn delete_player(&self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        let guild_id = guild_id.into();
        let node = self.get_node_for_guild(guild_id);

        if let Some((_, player)) = self.players.remove(&guild_id) {
            player.close()?;
        }

        node.http
            .delete_player(guild_id, &node.session_id.load())
            .await?;

        Ok(())
    }

    /// Deletes all stored player contexts.
    ///
    /// This is useful to put on the ready event, to close already open players in case the
    /// Lavalink server restarts.
    pub async fn delete_all_player_contexts(&self) -> LavalinkResult<()> {
        for guild_id in self.players.iter().map(|i| i.guild_id).collect::<Vec<_>>() {
            self.delete_player(guild_id).await?;
        }

        Ok(())
    }

    /// Request a raw player update.
    pub async fn update_player(
        &self,
        guild_id: impl Into<GuildId>,
        update_player: &http::UpdatePlayer,
        no_replace: bool,
    ) -> LavalinkResult<player::Player> {
        let guild_id = guild_id.into();
        let node = self.get_node_for_guild(guild_id);

        let result = node
            .http
            .update_player(guild_id, &node.session_id.load(), update_player, no_replace)
            .await?;

        if let Some(player) = self.get_player_context(guild_id) {
            player.update_player_data(result.clone())?;
        }

        Ok(result)
    }

    /// Resolves audio tracks for use with the `update_player` endpoint.
    ///
    /// # Parameters
    ///
    /// - `identifier`: A track identifier.
    ///  - Can be a url: "https://youtu.be/watch?v=DrM2lo6B04I"
    ///  - A unique identifier: "DrM2lo6B04I"
    ///  - A search: "
    pub async fn load_tracks(
        &self,
        guild_id: impl Into<GuildId>,
        identifier: &str,
    ) -> LavalinkResult<track::Track> {
        let guild_id = guild_id.into();
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.load_tracks(identifier).await?;

        Ok(result)
    }

    /// Decode a single track into its info.
    ///
    /// # Parameters
    ///
    /// - `track`: base64 encoded track data.
    pub async fn decode_track(
        &self,
        guild_id: impl Into<GuildId>,
        track: &str,
    ) -> LavalinkResult<track::TrackData> {
        let guild_id = guild_id.into();
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.decode_track(track).await?;

        Ok(result)
    }

    /// Decode multiple tracks into their info.
    ///
    /// # Parameters
    ///
    /// - `tracks`: base64 encoded tracks.
    pub async fn decode_tracks(
        &self,
        guild_id: impl Into<GuildId>,
        tracks: &[String],
    ) -> LavalinkResult<Vec<track::TrackData>> {
        let guild_id = guild_id.into();
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.decode_tracks(tracks).await?;

        Ok(result)
    }

    /// Request Lavalink server version.
    pub async fn request_version(&self, guild_id: impl Into<GuildId>) -> LavalinkResult<String> {
        let guild_id = guild_id.into();
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.version().await?;

        Ok(result)
    }

    /// Request Lavalink statistics.
    ///
    /// NOTE: The frame stats will never be returned.
    pub async fn request_stats(
        &self,
        guild_id: impl Into<GuildId>,
    ) -> LavalinkResult<events::Stats> {
        let guild_id = guild_id.into();
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.stats().await?;

        Ok(result)
    }

    /// Request Lavalink server information.
    pub async fn request_info(&self, guild_id: impl Into<GuildId>) -> LavalinkResult<http::Info> {
        let guild_id = guild_id.into();
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.info().await?;

        Ok(result)
    }

    /// Returns the player for the guild.
    pub async fn request_player(
        &self,
        guild_id: impl Into<GuildId>,
    ) -> LavalinkResult<player::Player> {
        let guild_id = guild_id.into();
        let node = self.get_node_for_guild(guild_id);

        let result = node
            .http
            .get_player(guild_id, &node.session_id.load())
            .await?;

        Ok(result)
    }

    /// Returns all players from the Node bound to the guild.
    pub async fn request_all_players(
        &self,
        guild_id: impl Into<GuildId>,
    ) -> LavalinkResult<Vec<player::Player>> {
        let guild_id = guild_id.into();
        let node = self.get_node_for_guild(guild_id);

        let result = node.http.get_players(&node.session_id.load()).await?;

        Ok(result)
    }
}
