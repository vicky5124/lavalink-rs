pub mod model;
pub mod gateway;
pub mod error;

use model::*;
use gateway::*;
use error::LavalinkError;

use std::{
    sync::Arc,
    time::Duration,
    collections::HashMap,
    cmp::{
        min,
        max,
    },
};

#[cfg(feature = "tokio-02-marker")]
use reqwest_compat as reqwest;
#[cfg(feature = "tokio-02-marker")]
use tokio_compat as tokio;
#[cfg(feature = "tokio-02-marker")]
use async_tungstenite_compat as async_tungstenite;

use serenity::model::guild::Region;
use songbird::ConnectionInfo;

use http::Request;

use reqwest::{
    Client as ReqwestClient,
    header::*,
    Url,
    Error as ReqwestError,
};

#[cfg(all(feature = "native-marker", not(feature = "tokio-02-marker")))]
use tokio_native_tls::TlsStream;

#[cfg(all(feature = "rustls-marker", not(feature = "tokio-02-marker")))]
use tokio_rustls::client::TlsStream;

#[cfg(all(feature = "native-marker", feature = "tokio-02-marker"))]
use tokio_native_tls_compat::TlsStream;

#[cfg(all(feature = "rustls-marker", feature = "tokio-02-marker"))]
use tokio_rustls_compat::client::TlsStream;


#[cfg(feature = "tokio-02-marker")]
use tokio::time::delay_for as sleep;

#[cfg(not(feature = "tokio-02-marker"))]
use tokio::time::sleep;

use tokio::{
    sync::Mutex,
    net::TcpStream,
};

use regex::Regex;

use futures::stream::{
    SplitSink,
    StreamExt,
};


use async_tungstenite::{
    tungstenite::{
        error::Error as TungsteniteError,
        Message as TungsteniteMessage,
    },
    stream::Stream,
    WebSocketStream,
    tokio::{
        connect_async,
        TokioAdapter,
    },
};

pub const EQ_BASE: [f64; 15] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
pub const EQ_BOOST: [f64; 15] = [-0.075, 0.125, 0.125, 0.1, 0.1, 0.05, 0.075, 0.0, 0.0, 0.0, 0.0, 0.0, 0.125, 0.15, 0.05];
pub const EQ_METAL: [f64; 15] = [0.0, 0.1, 0.1, 0.15, 0.13, 0.1, 0.0, 0.125, 0.175, 0.175, 0.125, 0.125, 0.1, 0.075, 0.0];
pub const EQ_PIANO: [f64; 15] = [-0.25, -0.25, -0.125, 0.0, 0.25, 0.25, 0.0, -0.25, -0.25, 0.0, 0.0, 0.5, 0.25, -0.025, 0.0];

#[cfg(not(feature = "tokio-02-marker"))]
pub type WsStream = WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>;
#[cfg(feature = "tokio-02-marker")]
pub type WsStream = WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>;

pub type WebsocketConnection = Arc<Mutex<WsStream>>;

#[derive(Default)]
pub struct LavalinkClient {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub shard_count: u64,
    pub bot_id: UserId,
    pub is_ssl: bool,

    pub headers: Option<HeaderMap>,
    pub socket_write: Option<SplitSink<WsStream, TungsteniteMessage>>,
    //pub socket_read: Option<SplitStream<WsStream>>,

    pub rest_uri: String,
    pub socket_uri: String,

    // Unused
    _region: Option<Region>,
    _identifier: Option<String>,
    //_shard_id: Option<ShardId>,

    pub nodes: HashMap<u64, Node>,
    pub loops: Vec<u64>,
}

#[derive(Default)]
pub struct PlayParameters {
    pub track: Track,
    pub replace: bool,
    pub start: u64,
    pub finish: u64,
    pub guild_id: u64,
    pub requester: Option<UserId>,
}

impl PlayParameters {
    /// Starts playing the track.
    pub async fn start(self, socket: &mut SplitSink<WsStream, TungsteniteMessage>) -> LavalinkResult<()> {
        let payload = crate::model::Play {
            track: self.track.track,
            no_replace: !self.replace,
            start_time: self.start,
            end_time: if self.finish == 0 { None } else { Some(self.finish) },
        };

        crate::model::SendOpcode::Play(payload).send(self.guild_id, socket).await?;


        Ok(())
    }

    pub async fn queue(self, client: Arc<Mutex<LavalinkClient>>) -> LavalinkResult<()> {
        let track = crate::model::TrackQueue {
            track: self.track,
            start_time: self.start,
            end_time: if self.finish == 0 { None } else { Some(self.finish) },
            requester: self.requester,
        };

        let client_clone = Arc::clone(&client);
        let mut client = client.lock().await;

        if !client.loops.contains(&self.guild_id) {
            let guild_id = self.guild_id;

            client.nodes.insert(guild_id, Node::default());
            client.loops.push(guild_id);

            let node = client.nodes.get_mut(&guild_id).unwrap();
            node.queue.push(track.clone());

            drop(client);

            tokio::spawn(async move {
                loop {
                    let mut client = client_clone.lock().await;
                    if let Some(node) = client.nodes.get_mut(&guild_id) {
                        if !node.queue.is_empty() && node.now_playing.is_none() {
                            let track = node.queue[0].clone();

                            node.now_playing = Some(node.queue[0].clone());

                            let payload = crate::model::Play {
                                track: track.track.track.clone(), // track
                                no_replace: false,
                                start_time: track.start_time,
                                end_time: track.end_time,
                            };

                            if let Some(ref mut socket) = &mut client.socket_write {
                                if let Err(why) = crate::model::SendOpcode::Play(payload).send(guild_id, socket).await {
                                    eprintln!("Error playing queue on guild {} -> {}", guild_id, why);
                                }
                            } else {
                                eprintln!("Error playing queue on guild {} -> No Socket Found", guild_id);
                            }
                        }
                    } else {
                        break
                    }

                    drop(client);
                    sleep(Duration::from_secs(1)).await;
                }
            });
        } else {
            let node = client.nodes.get_mut(&self.guild_id).unwrap();
            node.queue.push(track);
        }


        Ok(())
    }

    /// Sets the person that requested the song
    pub fn requester(mut self, requester: impl Into<UserId>) -> Self {
        self.requester = Some(requester.into());
        self
    }

    /// Sets if the current playing track should be replaced with this new one.
    pub fn replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }

    /// Sets the time the track will start at.
    pub fn start_time(mut self, start: Duration) -> Self {
        self.start = start.as_millis() as u64;
        self
    }

    /// Sets the time the track will finish at.
    pub fn finish_time(mut self, finish: Duration) -> Self {
        self.finish = finish.as_millis() as u64;
        self
    }
}

impl LavalinkClient {
    /// Builds a basic uninitialized LavalinkClient.
    pub fn new<U: Into<UserId>>(bot_id: U) -> Self {
        let mut client = LavalinkClient::default();
        client.host = "localhost".to_string();
        client.port = 2333;
        client.password = "youshallnotpass".to_string();
        client.shard_count = 1;
        client.bot_id = bot_id.into();
        client
    }

    /// Sets the host.
    ///
    /// DEFAULT: `localhost`
    pub fn set_host(&mut self, host: impl ToString) {
        self.host = host.to_string();
    }

    /// Sets the port.
    ///
    /// DEFAULT: `2333`
    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    /// Sets the number of shards.
    ///
    /// DEFAULT: `1`
    pub fn set_shard_count(&mut self, shard_count: u64) {
        self.shard_count = shard_count;
    }

    /// Sets the ID of the bot.
    pub fn set_bot_id<U: Into<UserId>>(&mut self, bot_id: U) {
        self.bot_id = bot_id.into();
    }

    /// Sets if the lavalink server is behind SSL
    ///
    /// DEFAULT: `False`
    pub fn set_is_ssl(&mut self, is_ssl: bool) {
        self.is_ssl = is_ssl;
    }

    /// Sets the lavalink password.
    ///
    /// DEFAULT: `youshallnotpass`
    pub fn set_password(&mut self, password: impl ToString) {
        self.password = password.to_string();
    }

    /// Initializes the connection with the provided information.
    pub async fn initialize(mut self, handler: impl LavalinkEventHandler + Send + Sync + 'static) -> Result<Arc<Mutex<Self>>, TungsteniteError> {
        if self.is_ssl {
            self.socket_uri = format!("wss://{}:{}", &self.host, &self.port);
            self.rest_uri = format!("https://{}:{}", &self.host, &self.port);
        } else {
            self.socket_uri = format!("ws://{}:{}", &self.host, &self.port);
            self.rest_uri = format!("http://{}:{}", &self.host, &self.port);
        }

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", self.password.parse()?);
        headers.insert("Num-Shards", self.shard_count.to_string().parse()?);
        headers.insert("User-Id", self.bot_id.to_string().parse()?);

        self.headers = Some(headers);

        let url = Request::builder()
            .uri(&self.socket_uri)
            .header("Authorization", &self.password)
            .header("Num-Shards", &self.shard_count.to_string())
            .header("User-Id", &self.bot_id.to_string())
            .body(())
            .unwrap();

        let (ws_stream, _) = connect_async(url).await?;

        let (write, mut read) = ws_stream.split();
        self.socket_write = Some(write);
        //self.socket = Some(Arc::new(Mutex::new(ws_stream)));

        let client = Arc::new(Mutex::new(self));
        let client_clone = Arc::clone(&client);

        tokio::spawn(async move {
            while let Some(Ok(resp)) = read.next().await {
                match &resp {
                    TungsteniteMessage::Text(x) => {
                        if let Ok(base_event) = serde_json::from_str::<GatewayEvent>(&x) {
                            match base_event.op.as_str() {
                                "stats" => {
                                    if let Ok(stats) = serde_json::from_str::<Stats>(&x) {
                                        handler.stats(Arc::clone(&client), stats).await;
                                    }
                                },
                                "playerUpdate" => {
                                    if let Ok(player_update) = serde_json::from_str::<PlayerUpdate>(&x) {
                                        {
                                            let mut client_lock = client.lock().await;

                                            if let Some(node) = client_lock.nodes.get_mut(&player_update.guild_id) {
                                                if let Some(mut current_track) = node.now_playing.as_mut() {
                                                    let mut info = current_track.track.info.as_mut().unwrap().clone();
                                                    info.position = player_update.state.position as u64;
                                                    current_track.track.info = Some(info);
                                                }
                                            }
                                        }
                                        handler.player_update(Arc::clone(&client), player_update).await;
                                    }
                                },
                                "event" => {
                                    match base_event.event_type.unwrap().as_str() {
                                        "TrackStartEvent" => {
                                            if let Ok(track_start) = serde_json::from_str::<TrackStart>(&x) {
                                                handler.track_start(Arc::clone(&client), track_start).await;
                                            }
                                        },
                                        "TrackEndEvent" => {
                                            let client_clone = Arc::clone(&client);
                                            if let Ok(track_finish) = serde_json::from_str::<TrackFinish>(&x) {
                                                if track_finish.reason == "FINISHED" {
                                                    let mut client = client_clone.lock().await;

                                                    if let Some(node) = client.nodes.get_mut(&track_finish.guild_id) {
                                                        node.queue.remove(0);
                                                        node.now_playing = None;
                                                    }
                                                }

                                                handler.track_finish(client_clone, track_finish).await;
                                            }
                                        },
                                        _ => (),
                                    }
                                },
                                _ => (),
                            }
                        }
                    },
                    _ => (),
                }
            }
        });

        Ok(client_clone)
    }

    /// Alias to `initialize()`
    pub async fn init(self, handler: impl LavalinkEventHandler + Send + Sync + 'static) -> Result<Arc<Mutex<Self>>, TungsteniteError> {
        self.initialize(handler).await
    }

    /// Returns the tracks from the URL or query provided.
    pub async fn get_tracks(&self, query: impl ToString) -> Result<Tracks, ReqwestError> {
        let reqwest = ReqwestClient::new();
        let url = Url::parse_with_params(&format!("{}/loadtracks", &self.rest_uri), &[("identifier", &query.to_string())]).expect("The query cannot be formated to a url.");

        let resp = reqwest.get(url)
            .headers(self.headers.clone().unwrap())
            .send()
            .await?
            .json::<Tracks>()
            .await?;

        Ok(resp)
    }

    /// Will automatically search the query if it's not a valid URL.
    pub async fn auto_search_tracks(&self, query: impl ToString) -> Result<Tracks, ReqwestError> {
        let r = Regex::new(r"https?://(?:www\.)?.+").unwrap();
        if r.is_match(&query.to_string()) {
            self.get_tracks(query.to_string()).await
        } else {
            self.get_tracks(format!("ytsearch:{}", query.to_string())).await
        }
    }

    /// Returns tracks from the search query.
    pub async fn search_tracks(&self, query: impl ToString) -> Result<Tracks, ReqwestError> {
        self.get_tracks(format!("ytsearch:{}", query.to_string())).await
    }

    /// Creates a lavalink session on the specified guild.
    pub async fn create_session(&mut self, guild_id: impl Into<GuildId>, connection_info: &ConnectionInfo) -> LavalinkResult<()> {
        let guild_id = guild_id.into();

        let socket = if let Some(x) = &mut self.socket_write { x } else {
            return Err(LavalinkError::NoWebsocket);
        };

        let guild_id_str = guild_id.0.to_string();

        let token = &connection_info.token;
        if token.is_empty() {
            return Err(LavalinkError::MissingHandlerToken)
        }

        let endpoint = &connection_info.endpoint;
        if endpoint.is_empty() {
            return Err(LavalinkError::MissingHandlerEndpoint)
        }

        let session_id = &connection_info.session_id;
        if session_id.is_empty() {
            return Err(LavalinkError::MissingHandlerSessionId)
        }

        
        let event = crate::model::Event {
            token: token.to_string(),
            endpoint: endpoint.to_string(),
            guild_id: guild_id_str,
        };

        let payload = crate::model::VoiceUpdate {
            session_id: session_id.to_string(),
            event,
        };

        crate::model::SendOpcode::VoiceUpdate(payload).send(guild_id, socket).await?;

        Ok(())
    }

    /// Constructor for playing a track.
    pub fn play(guild_id: impl Into<GuildId>, track: Track) -> PlayParameters {
        let mut p = PlayParameters::default();
        p.track = track;
        p.guild_id = guild_id.into().0;
        p
    }

    /// Destroys the current player.
    /// When this is ran, `create_session()` needs to be ran again.
    pub async fn destroy(&mut self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        let guild_id = guild_id.into();

        let socket = if let Some(x) = &mut self.socket_write { x } else {
            return Err(LavalinkError::NoWebsocket);
        };

        if let Some(node) = self.nodes.get_mut(&guild_id.0) {
            node.now_playing = None;

            if !node.queue.is_empty() {
                node.queue.remove(0);
            }
        }

        crate::model::SendOpcode::Destroy.send(guild_id, socket).await?;

        Ok(())
    }

    /// Stops the current player.
    pub async fn stop(&mut self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        let socket = if let Some(x) = &mut self.socket_write { x } else {
            return Err(LavalinkError::NoWebsocket);
        };

        crate::model::SendOpcode::Stop.send(guild_id, socket).await?;

        Ok(())
    }

    /// Skips the current playing track to the next item on the queue.
    ///
    /// If nothing is in the queue, the currently playing track will keep playing.
    /// Check if the queue is empty and run `stop()` if that's the case.
    pub async fn skip(&mut self, guild_id: impl Into<GuildId>) -> Option<TrackQueue> {
        let node = self.nodes.get_mut(&guild_id.into().0)?;

        node.now_playing = None;

        if node.queue.is_empty() {
            None
        } else {
            Some(node.queue.remove(0))
        }
    }

    /// Sets the pause status.
    pub async fn set_pause(&mut self, guild_id: impl Into<GuildId>, pause: bool) -> LavalinkResult<()> {
        let socket = if let Some(x) = &mut self.socket_write { x } else {
            return Err(LavalinkError::NoWebsocket);
        };

        let payload = crate::model::Pause {
            pause,
        };

        crate::model::SendOpcode::Pause(payload).send(guild_id, socket).await?;

        Ok(())
    }

    /// Sets pause status to `True`
    pub async fn pause(&mut self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        self.set_pause(guild_id, true).await
    }

    /// Sets pause status to `False`
    pub async fn resume(&mut self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        self.set_pause(guild_id, false).await
    }

    /// Jumps to a specific time in the currently playing track.
    pub async fn seek(&mut self, guild_id: impl Into<GuildId>, time: Duration) -> LavalinkResult<()> {
        let socket = if let Some(x) = &mut self.socket_write { x } else {
            return Err(LavalinkError::NoWebsocket);
        };

        let payload = crate::model::Seek {
            position: time.as_millis() as u64,
        };

        crate::model::SendOpcode::Seek(payload).send(guild_id, socket).await?;

        Ok(())
    }

    /// Alias to `seek()`
    pub async fn jump_to_time(&mut self, guild_id: impl Into<GuildId>, time: Duration) -> LavalinkResult<()> {
        self.seek(guild_id, time).await
    }

    /// Alias to `seek()`
    pub async fn scrub(&mut self, guild_id: impl Into<GuildId>, time: Duration) -> LavalinkResult<()> {
        self.seek(guild_id, time).await
    }

    /// Sets the volume of the player.
    pub async fn volume(&mut self, guild_id: impl Into<GuildId>, volume: u16) -> LavalinkResult<()> {
        let socket = if let Some(x) = &mut self.socket_write { x } else {
            return Err(LavalinkError::NoWebsocket);
        };

        let good_volume = max(min(volume, 1000), 0);

        let payload = crate::model::Volume {
            volume: good_volume,
        };

        crate::model::SendOpcode::Volume(payload).send(guild_id, socket).await?;

        Ok(())
    }

    /// Sets all equalizer levels.
    ///
    /// There are 15 bands (0-14) that can be changed.
    /// The floating point value is the multiplier for the given band. The default value is 0.
    /// Valid values range from -0.25 to 1.0, where -0.25 means the given band is completely muted, and 0.25 means it is doubled.
    /// Modifying the gain could also change the volume of the output.
    pub async fn equalize_all(&mut self, guild_id: impl Into<GuildId>, bands: [f64; 15]) -> LavalinkResult<()> {
        let socket = if let Some(x) = &mut self.socket_write { x } else {
            return Err(LavalinkError::NoWebsocket);
        };

        let bands = bands.iter().enumerate().map(|(index, i)| {
            crate::model::Band {
                band: index as u8,
                gain: *i,
            }
        }).collect::<Vec<_>>();

        let payload = crate::model::Equalizer {
            bands,
        };

        crate::model::SendOpcode::Equalizer(payload).send(guild_id, socket).await?;

        Ok(())
    }

    /// Equalizes a specific band.
    pub async fn equalize_band(&mut self, guild_id: impl Into<GuildId>, band: crate::model::Band) -> LavalinkResult<()> {
        let socket = if let Some(x) = &mut self.socket_write { x } else {
            return Err(LavalinkError::NoWebsocket);
        };

        let payload = crate::model::Equalizer {
            bands: vec![band],
        };

        crate::model::SendOpcode::Equalizer(payload).send(guild_id, socket).await?;

        Ok(())
    }

    /// Resets all equalizer levels.
    pub async fn equalize_reset(&mut self, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
        let socket = if let Some(x) = &mut self.socket_write { x } else {
            return Err(LavalinkError::NoWebsocket);
        };

        let bands = (0..=14).map(|i| {
            crate::model::Band {
                band: i as u8,
                gain: 0.,
            }
        }).collect::<Vec<_>>();

        let payload = crate::model::Equalizer {
            bands,
        };

        crate::model::SendOpcode::Equalizer(payload).send(guild_id, socket).await?;

        Ok(())
    }
}
