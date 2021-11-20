//! A Lavalink and Andesite API wrapper library for every tokio based discord bot library.

#[cfg(feature = "tracing-log")]
#[macro_use]
extern crate tracing;

#[cfg(feature = "normal-log")]
#[macro_use]
extern crate log;

/// Builder structures
pub mod builders;
/// Library's errors
pub mod error;
mod event_loops;
/// Gateway events
pub mod gateway;
/// Library models
pub mod model;
#[cfg(feature = "discord-gateway")]
/// Voice connection handling
pub mod voice;

/// Re-export to be used with the event handler.
pub use async_trait::async_trait;
/// Re-export to be used with the Node data.
pub use typemap_rev;

use builders::*;
#[cfg(feature = "discord-gateway")]
use error::LavalinkError;
use error::LavalinkResult;

#[cfg(feature = "discord-gateway")]
use event_loops::discord_event_loop;
use event_loops::lavalink_event_loop;

use gateway::LavalinkEventHandler;
use model::*;

use std::{
    cmp::{max, min},
    sync::Arc,
    time::Duration,
};

#[cfg(feature = "songbird")]
use songbird_dep::ConnectionInfo as SongbirdConnectionInfo;

use http::Request;

use reqwest::{header::*, Client as ReqwestClient, Url};

#[cfg(feature = "native")]
use tokio_native_tls::TlsStream;
#[cfg(feature = "rustls")]
use tokio_rustls::client::TlsStream;

use tokio::{net::TcpStream, sync::Mutex};

use regex::Regex;

use futures::stream::{SplitSink, StreamExt};

use async_tungstenite::{
    stream::Stream,
    tokio::{connect_async, TokioAdapter},
    tungstenite::Message as TungsteniteMessage,
    WebSocketStream,
};

#[cfg(feature = "discord-gateway")]
use tokio::sync::mpsc;

use dashmap::{DashMap, DashSet};

/// All 0's equalizer preset. Default.
pub const EQ_BASE: [f64; 15] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

/// Basic boost equalizer with higher lows and highs.
pub const EQ_BOOST: [f64; 15] = [
    -0.075, 0.125, 0.125, 0.1, 0.1, 0.05, 0.075, 0.0, 0.0, 0.0, 0.0, 0.0, 0.125, 0.15, 0.05,
];
/// Equalizer preset for most metal music.
pub const EQ_METAL: [f64; 15] = [
    0.0, 0.1, 0.1, 0.15, 0.13, 0.1, 0.0, 0.125, 0.175, 0.175, 0.125, 0.125, 0.1, 0.075, 0.0,
];
/// Equalizer preset for piano and classical.
pub const EQ_PIANO: [f64; 15] = [
    -0.25, -0.25, -0.125, 0.0, 0.25, 0.25, 0.0, -0.25, -0.25, 0.0, 0.0, 0.5, 0.25, -0.025, 0.0,
];

pub type WsStream =
    WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>;

/// NOTE: All fields are public for those who want to do their own implementation of things, you
/// should not be touching them if you don't know what you are doing.
pub struct LavalinkClientInner {
    //pub socket_uri: String,
    pub rest_uri: String,
    pub headers: HeaderMap,

    /// The sender websocket split.
    pub socket_write: SplitSink<WsStream, TungsteniteMessage>,
    // cannot be cloned, and cannot be behind a lock
    // because it would always be open by the event loop.
    //pub socket_read: SplitStream<WsStream>,

    //_shard_id: Option<ShardId>,
    pub nodes: Arc<DashMap<u64, Node>>,
    pub loops: Arc<DashSet<u64>>,

    #[cfg(feature = "discord-gateway")]
    pub discord_gateway_data: Arc<Mutex<DiscordGatewayData>>,
    // Unused
    //_region: Option<Region>,
    //_identifier: Option<String>,
}

#[cfg(feature = "discord-gateway")]
pub struct DiscordGatewayData {
    pub shard_count: u64,
    pub bot_id: UserId,
    pub bot_token: String,
    pub wait_time: Duration,
    pub headers: HeaderMap,
    pub sender: mpsc::UnboundedSender<String>,
    pub connections: Arc<DashMap<GuildId, ConnectionInfo>>,
    pub socket_uri: &'static str,
}

/// A Client for Lavalink.
///
/// This structure is behind `Arc`, so it's clone and thread safe.
///
/// The inner field is public for those who want to tinker with it manually.
#[derive(Clone)]
pub struct LavalinkClient {
    /// Field is public for those who want to do their own implementation of things.
    pub inner: Arc<Mutex<LavalinkClientInner>>,
}

impl LavalinkClient {
    /// Builds the Client connection.
    pub async fn new(
        builder: &LavalinkClientBuilder,
        handler: impl LavalinkEventHandler + Send + Sync + 'static,
    ) -> LavalinkResult<Self> {
        let (lavalink_socket_write, lavalink_socket_read, lavalink_headers, lavalink_rest_uri) = {
            let socket_uri;
            let rest_uri;

            if builder.is_ssl {
                socket_uri = format!("wss://{}:{}", &builder.host, builder.port);
                rest_uri = format!("https://{}:{}", &builder.host, builder.port);
            } else {
                socket_uri = format!("ws://{}:{}", &builder.host, builder.port);
                rest_uri = format!("http://{}:{}", &builder.host, builder.port);
            }

            let mut headers = HeaderMap::new();
            headers.insert("Authorization", builder.password.parse()?);
            headers.insert("Num-Shards", builder.shard_count.to_string().parse()?);
            headers.insert("User-Id", builder.bot_id.to_string().parse()?);
            headers.insert(
                "Client-Name",
                concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
                    .to_owned()
                    .parse()?,
            );

            let mut url_builder = Request::builder();

            {
                let ref_headers = url_builder.headers_mut().unwrap();
                *ref_headers = headers.clone();
            }

            let url = url_builder.uri(&socket_uri).body(()).unwrap();

            let (ws_stream, _) = connect_async(url).await?;
            let split = ws_stream.split();

            (split.0, split.1, headers, rest_uri)
        };

        #[cfg(feature = "discord-gateway")]
        let (discord_socket_uri, discord_headers) = {
            let socket_uri = "wss://gateway.discord.gg/?v=9&encoding=json";

            let mut headers = HeaderMap::new();
            headers.insert(
                "Authorization",
                format!("Bot {}", builder.bot_token).parse()?,
            );
            headers.insert("bot", "True".to_string().parse()?);
            headers.insert("Content-type", "application/json".to_string().parse()?);

            (socket_uri, headers)
        };

        #[cfg(feature = "discord-gateway")]
        let discord_gateway_data = {
            Arc::new(Mutex::new(DiscordGatewayData {
                shard_count: builder.shard_count,
                bot_id: builder.bot_id,
                bot_token: builder.bot_token.to_string(),
                wait_time: builder.gateway_start_wait_time,
                headers: discord_headers,
                sender: mpsc::unbounded_channel().0,
                connections: Arc::new(DashMap::new()),
                socket_uri: discord_socket_uri,
            }))
        };

        let client_inner = LavalinkClientInner {
            headers: lavalink_headers,
            socket_write: lavalink_socket_write,
            rest_uri: lavalink_rest_uri,
            nodes: Arc::new(DashMap::new()),
            loops: Arc::new(DashSet::new()),
            #[cfg(feature = "discord-gateway")]
            discord_gateway_data,
        };

        let client = Self {
            inner: Arc::new(Mutex::new(client_inner)),
        };

        let client_clone = client.clone();

        tokio::spawn(async move {
            debug!("Starting lavalink event loop.");
            lavalink_event_loop(lavalink_socket_read, handler, client_clone).await;
            error!("Event loop ended unexpectedly.");
        });

        #[cfg(feature = "discord-gateway")]
        if builder.start_gateway {
            let client_clone = client.clone();
            let token = builder.bot_token.clone();
            let wait_time = builder.gateway_start_wait_time.clone();

            tokio::spawn(async move {
                debug!("Starting discord event loop.");
                discord_event_loop(client_clone, &token, wait_time).await;
                error!("Event loop ended unexpectedly.");
            });
        }

        Ok(client)
    }

    /// Returns a builder to be used to create a Client.
    ///
    /// ```rust
    /// struct LavalinkHandler;
    ///
    /// #[async_trait]
    /// impl LavalinkEventHandler for LavalinkHandler {
    ///     async fn track_start(&self, _client: LavalinkClient, event: TrackStart) {
    ///         info!("Track started!\nGuild: {}", event.guild_id);
    ///     }
    ///     async fn track_finish(&self, _client: LavalinkClient, event: TrackFinish) {
    ///         info!("Track finished!\nGuild: {}", event.guild_id);
    ///     }
    /// }
    ///     
    /// let lavalink_client = LavalinkClient::builder(bot_id)
    ///     .set_host("127.0.0.1")
    ///     .set_password(env::var("LAVALINK_PASSWORD").unwrap_or("youshallnotpass".to_string()))
    ///     .build(LavalinkHandler)
    ///     .await?;
    /// ```
    #[cfg(feature = "discord-gateway")]
    pub fn builder(
        user_id: impl Into<UserId>,
        bot_token: impl Into<String>,
    ) -> LavalinkClientBuilder {
        LavalinkClientBuilder::new(user_id, bot_token)
    }
    #[cfg(not(feature = "discord-gateway"))]
    pub fn builder(user_id: impl Into<UserId>) -> LavalinkClientBuilder {
        LavalinkClientBuilder::new(user_id)
    }

    /// Start the discord gateway, if it has stopped, or it never started because the client builder was
    /// configured that way.
    ///
    /// If wait_time is passed, it will override the previosuly configured wait time.
    #[cfg(feature = "discord-gateway")]
    pub async fn start_discord_gateway(&self, wait_time: Option<Duration>) {
        let client_clone = self.clone();
        let token = self
            .discord_gateway_data()
            .await
            .lock()
            .await
            .bot_token
            .clone();
        let wait_time = if let Some(t) = wait_time {
            t
        } else {
            self.discord_gateway_data()
                .await
                .lock()
                .await
                .wait_time
                .clone()
        };

        tokio::spawn(async move {
            debug!("Starting discord event loop.");
            discord_event_loop(client_clone, &token, wait_time).await;
            error!("Event loop ended unexpectedly.");
        });
    }

    /// Returns the tracks from the URL or query provided.
    pub async fn get_tracks(&self, query: impl ToString) -> LavalinkResult<Tracks> {
        let client = self.inner.lock().await;

        let reqwest = ReqwestClient::new();
        let url = Url::parse_with_params(
            &format!("{}/loadtracks", &client.rest_uri),
            &[("identifier", &query.to_string())],
        )
        .expect("The query cannot be formatted to a url.");

        let resp = reqwest
            .get(url)
            .headers(client.headers.clone())
            .send()
            .await?
            .json::<Tracks>()
            .await?;

        Ok(resp)
    }

    /// Will automatically search the query on youtube if it's not a valid URL.
    pub async fn auto_search_tracks(&self, query: impl ToString) -> LavalinkResult<Tracks> {
        let r = Regex::new(r"https?://(?:www\.)?.+").unwrap();
        if r.is_match(&query.to_string()) {
            self.get_tracks(query.to_string()).await
        } else {
            self.get_tracks(format!("ytsearch:{}", query.to_string()))
                .await
        }
    }

    /// Returns tracks from the search query.
    /// Uses youtube to search.
    pub async fn search_tracks(&self, query: impl ToString) -> LavalinkResult<Tracks> {
        self.get_tracks(format!("ytsearch:{}", query.to_string()))
            .await
    }

    /// Decodes a track to it's information
    pub async fn decode_track(&self, track: impl ToString) -> LavalinkResult<Info> {
        let client = self.inner.lock().await;

        let reqwest = ReqwestClient::new();
        let url = Url::parse_with_params(
            &format!("{}/decodetrack", &client.rest_uri),
            &[("track", &track.to_string())],
        )
        .expect("The query cannot be formatted to a url.");

        let resp = reqwest
            .get(url)
            .headers(client.headers.clone())
            .send()
            .await?
            .json::<Info>()
            .await?;

        Ok(resp)
    }

    /// Creates a lavalink session on the specified guild.
    ///
    /// This also creates a Node and inserts it. The node is not added on loops unless
    /// Play::queue() is ran.
    #[cfg(feature = "songbird")]
    pub async fn create_session_with_songbird(
        &self,
        connection_info: &SongbirdConnectionInfo,
    ) -> LavalinkResult<()> {
        let event = crate::model::Event {
            token: connection_info.token.to_string(),
            endpoint: connection_info.endpoint.to_string(),
            guild_id: connection_info.guild_id.0.to_string(),
        };

        let payload = crate::model::VoiceUpdate {
            session_id: connection_info.session_id.to_string(),
            event,
        };

        let mut client = self.inner.lock().await;

        crate::model::SendOpcode::VoiceUpdate(payload)
            .send(connection_info.guild_id, &mut client.socket_write)
            .await?;

        if !client.nodes.contains_key(&connection_info.guild_id.0) {
            client
                .nodes
                .insert(connection_info.guild_id.0, Node::default());
        }

        Ok(())
    }
    #[cfg(feature = "discord-gateway")]
    pub async fn create_session(&self, connection_info: &ConnectionInfo) -> LavalinkResult<()> {
        let token = connection_info
            .token
            .as_ref()
            .ok_or(LavalinkError::MissingConnectionField("token"))?
            .to_string();
        let endpoint = connection_info
            .endpoint
            .as_ref()
            .ok_or(LavalinkError::MissingConnectionField("endpoint"))?
            .to_string();
        let guild_id = connection_info
            .guild_id
            .as_ref()
            .ok_or(LavalinkError::MissingConnectionField("guild_id"))?
            .to_string();
        let session_id = connection_info
            .session_id
            .as_ref()
            .ok_or(LavalinkError::MissingConnectionField("session_id"))?
            .to_string();

        let endpoint = if endpoint.starts_with("wss://") {
            endpoint.strip_prefix("wss://").unwrap().to_string()
        } else {
            endpoint
        };

        let event = crate::model::Event {
            token,
            endpoint,
            guild_id,
        };

        let payload = crate::model::VoiceUpdate { session_id, event };

        let mut client = self.inner.lock().await;

        crate::model::SendOpcode::VoiceUpdate(payload)
            .send(connection_info.guild_id.unwrap(), &mut client.socket_write)
            .await?;

        if !client
            .nodes
            .contains_key(&connection_info.guild_id.unwrap().0)
        {
            client
                .nodes
                .insert(connection_info.guild_id.unwrap().0, Node::default());
        }

        Ok(())
    }

    /// Constructor for playing a track.
    pub fn play(&self, guild_id: impl Into<GuildId>, track: Track) -> PlayParameters {
        PlayParameters {
            track,
            guild_id: guild_id.into().0,
            client: self.clone(),
            replace: false,
            start: 0,
            finish: 0,
            requester: None,
        }
    }

    /// Destroys the current player.
    /// When this is ran, `create_session()` needs to be ran again.
    ///
    /// This method does not remove the guild from the running event loops, nor does it clear the
    /// Node, this allows for reconnecting without losing data.
    /// If you are having issues with disconnecting and reconnecting the bot to a voice channel,
    /// remove the guild from the running event loops and reset the nodes.
    ///
    /// The running loops and the nodes can be obtained via `LavalinkClient::nodes()` and
    /// `LavalinkClient::loops()`
    ///
    /// ```rust,untested
    /// lavalink_client.destroy(guild_id).await?;
    ///
    /// {
    ///     let nodes = lavalink_client.nodes().await;
    ///     nodes.remove(&guild_id.0);
    ///     
    ///     let loops = lavalink_client.loops().await;
    ///     loops.remove(&guild_id.0);
    /// }
    /// ```
    pub async fn destroy(&self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        let guild_id = guild_id.into();

        let mut client = self.inner.lock().await;

        if let Some(mut node) = client.nodes.get_mut(&guild_id.0) {
            node.now_playing = None;

            if !node.queue.is_empty() {
                node.queue.remove(0);
            }
        }

        crate::model::SendOpcode::Destroy
            .send(guild_id, &mut client.socket_write)
            .await?;

        Ok(())
    }

    /// Stops the current player.
    pub async fn stop(&self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        let mut client = self.inner.lock().await;

        crate::model::SendOpcode::Stop
            .send(guild_id, &mut client.socket_write)
            .await?;

        Ok(())
    }

    /// Skips the current playing track to the next item on the queue.
    ///
    /// If nothing is in the queue, the currently playing track will keep playing.
    /// Check if the queue is empty and run `stop()` if that's the case.
    pub async fn skip(&self, guild_id: impl Into<GuildId>) -> Option<TrackQueue> {
        let client = self.inner.lock().await;

        let mut node = client.nodes.get_mut(&guild_id.into().0)?;

        node.now_playing = None;

        if node.queue.is_empty() {
            None
        } else {
            Some(node.queue.remove(0))
        }
    }

    /// Sets the pause status.
    pub async fn set_pause(&self, guild_id: impl Into<GuildId>, pause: bool) -> LavalinkResult<()> {
        let guild_id = guild_id.into().0;
        let payload = crate::model::Pause { pause };

        {
            let nodes = self.nodes().await;
            let node = nodes.get_mut(&guild_id);
            if let Some(mut n) = node {
                n.is_paused = pause;
            }
        }

        let mut client = self.inner.lock().await;

        crate::model::SendOpcode::Pause(payload)
            .send(guild_id, &mut client.socket_write)
            .await?;

        Ok(())
    }

    /// Sets pause status to `True`
    pub async fn pause(&self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        self.set_pause(guild_id, true).await
    }

    /// Sets pause status to `False`
    pub async fn resume(&self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        self.set_pause(guild_id, false).await
    }

    /// Jumps to a specific time in the currently playing track.
    pub async fn seek(&self, guild_id: impl Into<GuildId>, time: Duration) -> LavalinkResult<()> {
        let payload = crate::model::Seek {
            position: time.as_millis() as u64,
        };

        let mut client = self.inner.lock().await;

        crate::model::SendOpcode::Seek(payload)
            .send(guild_id, &mut client.socket_write)
            .await?;

        Ok(())
    }

    /// Alias to `seek()`
    pub async fn jump_to_time(
        &self,
        guild_id: impl Into<GuildId>,
        time: Duration,
    ) -> LavalinkResult<()> {
        self.seek(guild_id, time).await
    }

    /// Alias to `seek()`
    pub async fn scrub(&self, guild_id: impl Into<GuildId>, time: Duration) -> LavalinkResult<()> {
        self.seek(guild_id, time).await
    }

    /// Sets the volume of the player.
    pub async fn volume(&self, guild_id: impl Into<GuildId>, volume: u16) -> LavalinkResult<()> {
        let good_volume = max(min(volume, 1000), 0);

        let payload = crate::model::Volume {
            volume: good_volume,
        };

        let mut client = self.inner.lock().await;

        crate::model::SendOpcode::Volume(payload)
            .send(guild_id, &mut client.socket_write)
            .await?;

        Ok(())
    }

    /// Sets all equalizer levels.
    ///
    /// - There are 15 bands (0-14) that can be changed.
    /// - The floating point value is the multiplier for the given band.
    /// - The default value is 0.
    /// - Valid values range from -0.25 to 1.0, where -0.25 means the given band is completely muted, and 0.25 means it is doubled.
    /// - Modifying the gain could also change the volume of the output.
    pub async fn equalize_all(
        &self,
        guild_id: impl Into<GuildId>,
        bands: [f64; 15],
    ) -> LavalinkResult<()> {
        let bands = bands
            .iter()
            .enumerate()
            .map(|(index, i)| crate::model::Band {
                band: index as u8,
                gain: *i,
            })
            .collect::<Vec<_>>();

        let payload = crate::model::Equalizer { bands };

        let mut client = self.inner.lock().await;

        crate::model::SendOpcode::Equalizer(payload)
            .send(guild_id, &mut client.socket_write)
            .await?;

        Ok(())
    }

    /// Equalize a dynamic set of bands, rather than just one or all of them at once.
    ///
    /// Unmentioned bands will remain unmodified.
    pub async fn equalize_dynamic(
        &self,
        guild_id: impl Into<GuildId>,
        bands: Vec<Band>,
    ) -> LavalinkResult<()> {
        let payload = crate::model::Equalizer { bands };

        let mut client = self.inner.lock().await;

        crate::model::SendOpcode::Equalizer(payload)
            .send(guild_id, &mut client.socket_write)
            .await?;

        Ok(())
    }

    /// Equalizes a specific band.
    pub async fn equalize_band(
        &self,
        guild_id: impl Into<GuildId>,
        band: crate::model::Band,
    ) -> LavalinkResult<()> {
        let payload = crate::model::Equalizer { bands: vec![band] };

        let mut client = self.inner.lock().await;

        crate::model::SendOpcode::Equalizer(payload)
            .send(guild_id, &mut client.socket_write)
            .await?;

        Ok(())
    }

    /// Resets all equalizer levels.
    pub async fn equalize_reset(&self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        let bands = (0..=14)
            .map(|i| crate::model::Band {
                band: i as u8,
                gain: 0.,
            })
            .collect::<Vec<_>>();

        let payload = crate::model::Equalizer { bands };

        let mut client = self.inner.lock().await;

        crate::model::SendOpcode::Equalizer(payload)
            .send(guild_id, &mut client.socket_write)
            .await?;

        Ok(())
    }

    /// Obtains an atomic reference to the nodes
    pub async fn nodes(&self) -> Arc<DashMap<u64, Node>> {
        let client = self.inner.lock().await;
        client.nodes.clone()
    }

    /// Obtains an atomic reference to the running queue loops
    ///
    /// A node guild_id is added here the first time [`PlayParameters::queue`] is called.
    ///
    /// [`PlayParameters::queue`]: crate::builders::PlayParameters

    pub async fn loops(&self) -> Arc<DashSet<u64>> {
        let client = self.inner.lock().await;
        client.loops.clone()
    }

    #[cfg(feature = "discord-gateway")]
    pub async fn discord_gateway_data(&self) -> Arc<Mutex<DiscordGatewayData>> {
        self.inner.lock().await.discord_gateway_data.clone()
    }

    #[cfg(feature = "discord-gateway")]
    pub async fn discord_gateway_connections(&self) -> Arc<DashMap<GuildId, ConnectionInfo>> {
        self.inner
            .lock()
            .await
            .discord_gateway_data
            .lock()
            .await
            .connections
            .clone()
    }

    #[cfg(feature = "discord-gateway")]
    pub async fn join(
        &self,
        guild_id: impl Into<GuildId>,
        channel_id: impl Into<ChannelId>,
    ) -> LavalinkResult<ConnectionInfo> {
        crate::voice::join(self, guild_id, channel_id).await
    }

    #[cfg(feature = "discord-gateway")]
    pub async fn leave(&self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        crate::voice::leave(self, guild_id).await
    }
}
