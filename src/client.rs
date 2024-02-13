use crate::error::{LavalinkError, LavalinkResult};
use crate::model::*;
use crate::node;
use crate::player_context::*;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use arc_swap::ArcSwap;
use dashmap::DashMap;
use reqwest::{header::HeaderMap, Client as ReqwestClient};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
/// The main client, where everything gets done, from events to requests to management.
pub struct LavalinkClient {
    pub nodes: Arc<Vec<node::Node>>,
    pub players: Arc<DashMap<GuildId, PlayerContext>>,
    pub events: events::Events,
    tx: UnboundedSender<ClientMessage>,
    user_id: UserId,
    user_data: Arc<dyn std::any::Any + Send + Sync>,
}

enum ClientMessage {
    GetConnectionInfo(
        GuildId,
        std::time::Duration,
        oneshot::Sender<Result<player::ConnectionInfo, tokio::time::error::Elapsed>>,
    ),
    ServerUpdate(GuildId, String, Option<String>), // guild_id, token, endpoint
    StateUpdate(GuildId, Option<ChannelId>, UserId, String), // guild_id, channel_id, user_id, session_id
}

impl LavalinkClient {
    /// Create a new Lavalink Client.
    /// It also establish the connection(s) and start listening for events.
    ///
    /// # Parameters
    ///
    /// - `events`: The lavalink event handler.
    /// - `nodes`: List of nodes to connect to.
    pub async fn new(events: events::Events, nodes: Vec<node::NodeBuilder>) -> LavalinkClient {
        Self::new_with_data(events, nodes, Arc::new(())).await
    }

    /// Create a new Lavalink Client with custom user data.
    /// It also establish the connection(s) and start listening for events.
    ///
    /// # Parameters
    ///
    /// - `events`: The lavalink event handler.
    /// - `nodes`: List of nodes to connect to.
    /// - `user_data`: Set the data that will be accessible from anywhere with the client.
    pub async fn new_with_data<Data: std::any::Any + Send + Sync>(
        events: events::Events,
        nodes: Vec<node::NodeBuilder>,
        user_data: Arc<Data>,
    ) -> LavalinkClient {
        if nodes.is_empty() {
            panic!("At least one node must be provided.");
        }

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

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let client = LavalinkClient {
            user_id: built_nodes[0].user_id,
            nodes: Arc::new(built_nodes),
            players: Arc::new(DashMap::new()),
            events,
            tx,
            user_data,
        };

        for node in &*client.nodes {
            if let Err(why) = node.connect(client.clone()).await {
                error!("Failed to connect to the lavalink websocket: {}", why);
            }
        }

        tokio::spawn(LavalinkClient::handle_connection_info(client.clone(), rx));

        let lavalink_client = client.clone();

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

        client
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
        let mut connection_info = connection_info.into();
        connection_info.fix();

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
        self.create_player_context_with_data(guild_id, connection_info, Arc::new(()))
            .await
    }

    /// Creates a new player with context with custom user data.
    ///
    /// Calling this method is required to create the initial player, and be able to use the built-in queue.
    pub async fn create_player_context_with_data<Data: std::any::Any + Send + Sync>(
        &self,
        guild_id: impl Into<GuildId>,
        connection_info: impl Into<player::ConnectionInfo>,
        user_data: Arc<Data>,
    ) -> LavalinkResult<PlayerContext> {
        let guild_id = guild_id.into();
        let mut connection_info = connection_info.into();
        connection_info.fix();

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
            user_data,
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

    /// Get the custom data provided when creating the client.
    ///
    /// # Errors
    /// Returns `LavalinkError::InvalidDataType` if the type argument provided does not match the type of the data provided,
    /// or if no data was provided when creating the client.
    pub fn data<Data: Send + Sync + 'static>(&self) -> LavalinkResult<std::sync::Arc<Data>> {
        self.user_data
            .clone()
            .downcast()
            .map_err(|_| LavalinkError::InvalidDataType)
    }

    /// Method to handle the VOICE_SERVER_UPDATE event.
    pub fn handle_voice_server_update(
        &self,
        guild_id: impl Into<GuildId>,
        token: String,
        endpoint: Option<String>,
    ) {
        let _ = self.tx.send(ClientMessage::ServerUpdate(
            guild_id.into(),
            token,
            endpoint,
        ));
    }

    /// Method to handle the VOICE_STATE_UPDATE event.
    pub fn handle_voice_state_update(
        &self,
        guild_id: impl Into<GuildId>,
        channel_id: Option<impl Into<ChannelId>>,
        user_id: impl Into<UserId>,
        session_id: String,
    ) {
        let _ = self.tx.send(ClientMessage::StateUpdate(
            guild_id.into(),
            channel_id.map(|x| x.into()),
            user_id.into(),
            session_id,
        ));
    }

    /// Returns the connection information needed for creating a player.
    ///
    /// This methods requires that `handle_voice_server_update` and `handle_voice_state_update` be
    /// defined and handled inside their respective discord events.
    ///
    /// # Errors
    /// If the custom timeout was reached. This can happen if the bot never connected to the voice
    /// channel, or the events were not handled correctly, or the timeout was too short.
    pub async fn get_connection_info(
        &self,
        guild_id: impl Into<GuildId>,
        timeout: std::time::Duration,
    ) -> LavalinkResult<player::ConnectionInfo> {
        let (tx, rx) = oneshot::channel();

        let _ = self.tx.send(ClientMessage::GetConnectionInfo(
            guild_id.into(),
            timeout,
            tx,
        ));

        rx.await?.map_err(|_| LavalinkError::Timeout)
    }

    async fn handle_connection_info(self, mut rx: UnboundedReceiver<ClientMessage>) {
        let data: DashMap<GuildId, (Option<String>, Option<String>, Option<String>)> =
            DashMap::new();
        let channels: DashMap<GuildId, (UnboundedSender<()>, UnboundedReceiver<()>)> =
            DashMap::new();

        'outer: while let Some(x) = rx.recv().await {
            use ClientMessage::*;

            match x {
                GetConnectionInfo(guild_id, timeout, sender) => {
                    channels
                        .entry(guild_id)
                        .or_insert(tokio::sync::mpsc::unbounded_channel());

                    let inner_rx = &mut channels.get_mut(&guild_id).unwrap().1;

                    loop {
                        match tokio::time::timeout(timeout, inner_rx.recv()).await {
                            Err(x) => {
                                if let Some((Some(token), Some(endpoint), Some(session_id))) =
                                    data.get(&guild_id).map(|x| x.value().clone())
                                {
                                    {
                                        let _ = sender.send(Ok(player::ConnectionInfo {
                                            token: token.to_string(),
                                            endpoint: endpoint.to_string(),
                                            session_id: session_id.to_string(),
                                        }));
                                        continue 'outer;
                                    }
                                }

                                let _ = sender.send(Err(x));
                                continue 'outer;
                            }
                            Ok(x) => {
                                if x.is_none() {
                                    continue 'outer;
                                };

                                if let Some((Some(token), Some(endpoint), Some(session_id))) =
                                    data.get(&guild_id).map(|x| x.value().clone())
                                {
                                    {
                                        let _ = sender.send(Ok(player::ConnectionInfo {
                                            token: token.to_string(),
                                            endpoint: endpoint.to_string(),
                                            session_id: session_id.to_string(),
                                        }));
                                        continue 'outer;
                                    }
                                }
                            }
                        }
                    }
                }
                ServerUpdate(guild_id, token, endpoint) => {
                    channels
                        .entry(guild_id)
                        .or_insert(tokio::sync::mpsc::unbounded_channel());

                    let inner_tx = &mut channels.get_mut(&guild_id).unwrap().0;

                    let mut entry = data.entry(guild_id).or_insert((None, None, None));
                    let session_id = entry.value().2.clone();
                    *entry.value_mut() = (Some(token), endpoint, session_id);

                    let _ = inner_tx.send(());
                }
                StateUpdate(guild_id, channel_id, user_id, session_id) => {
                    channels
                        .entry(guild_id)
                        .or_insert(tokio::sync::mpsc::unbounded_channel());

                    let inner_tx = &mut channels.get_mut(&guild_id).unwrap().0;

                    if user_id != self.user_id {
                        continue 'outer;
                    }
                    if channel_id.is_none() {
                        data.remove(&guild_id);
                        channels.remove(&guild_id);
                        continue 'outer;
                    }

                    let mut entry = data.entry(guild_id).or_insert((None, None, None));
                    let token = entry.value().0.clone();
                    let endpoint = entry.value().1.clone();
                    *entry.value_mut() = (token, endpoint, Some(session_id));

                    let _ = inner_tx.send(());
                }
            }
        }
    }
}
