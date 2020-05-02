use std::{
    sync::Arc,
    fmt::Display,
};

use serenity::{
    model::{
        guild::Region,
        id::UserId,
    },
    voice::Handler,
    client::bridge::gateway::ShardId,
};

use http::Request;
use reqwest::{
    Client as ReqwestClient,
    header::*,
    Url,
    Error as ReqwestError,
};

use tokio_tls::TlsStream;
use tokio::{
    sync::Mutex,
    net::TcpStream,
};

use regex::Regex;
use serde::Deserialize;
use serde_json::json;

use futures::prelude::*;
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


pub type WebsocketConnection = Arc<Mutex<WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TokioAdapter<TokioAdapter<TcpStream>>>>>>>>;

#[derive(Debug, Deserialize)]
pub struct Tracks {
    #[serde(rename = "playlistInfo")]
    pub playlist_info: PlaylistInfo,

    #[serde(rename = "loadType")]
    pub load_type: String,

    pub tracks: Vec<Track>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistInfo {
    #[serde(rename = "selectedTrack")]
    pub selected_track: Option<u64>,

    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub track: String,
    pub info: Info,
}

#[derive(Debug, Deserialize)]
pub struct Info {
    #[serde(rename = "isSeekable")]
    pub is_seekable: bool,

    #[serde(rename = "isStream")]
    pub is_stream: bool,

    pub identifier: String,
    pub author: String,
    pub length: u128,
    pub position: u128,
    pub title: String,
    pub uri: String,
}

#[derive(Clone, Default)]
pub struct LavalinkClient {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub shard_count: Option<u64>,
    pub _region: Option<Region>,
    pub _identifier: Option<String>,
    pub _shard_id: Option<ShardId>,
    pub bot_id: UserId,
    pub is_ssl: bool,
    pub socket: Option<WebsocketConnection>,
    pub headers: Option<HeaderMap>,
    pub rest_uri: String,
    pub socket_uri: String,
}

#[derive(Default, Clone, Copy)]
pub struct PlayParameters<'a, 'b, 'c> {
    pub client: Option<&'a LavalinkClient>,
    pub handler: Option<&'b Handler>,
    pub track: Option<&'c Track>,
    pub replace: bool,
    pub start: u128,
    pub finish: u128,
}

impl<'a, 'b, 'c> PlayParameters<'a, 'b, 'c> {
    pub async fn start(self) -> Result<(), String> {
        let socket = if let Some(x) = &self.client.unwrap().socket { x } else {
            return Err("There is no initialized websocket.".to_string());
        };
        let guild_id = self.handler.unwrap().guild_id.0.to_string();

        let token = if let Some(x) = self.handler.unwrap().token.as_ref() { x } else {
            return Err("No `token` was found on the handler.".to_string());
        };
        let endpoint = if let Some(x) = self.handler.unwrap().endpoint.as_ref() { x } else {
            return Err("No `endpoint` was found on the handler.".to_string());
        };

        let event = json!({ "token" : &token, "guild_id" : &guild_id, "endpoint" : &endpoint });

        let session_id = if let Some(x) = self.handler.unwrap().session_id.as_ref() { x } else {
            return Err("No `session id` was found on the handler.".to_string());
        };

        let payload = json!({ "op" : "voiceUpdate", "guildId" : &guild_id, "sessionId" : &session_id, "event" : event });

        let formated_payload = if let Ok(x) = serde_json::to_string(&payload) { x } else {
            return Err("Invalid data was provided to the `voiceUpdate` json.".to_string());
        };

        {
            let mut ws = socket.lock().await;
            if let Err(why) = ws.send(TungsteniteMessage::text(formated_payload)).await {
                return Err(format!("There was an error sending the `voiceUpdate` payload => {:?}", why));
            };
        }

        let payload = if self.finish > 0 {
            json!({ "op" : "play", "guildId" : &guild_id, "track" : self.track.unwrap().track, "noReplace" : !self.replace, "startTime" : self.start.to_string(), "endTime" : self.finish.to_string()})
        } else {
            json!({ "op" : "play", "guildId" : &guild_id, "track" : self.track.unwrap().track, "noReplace" : !self.replace, "startTime" : self.start.to_string()})
        };

        let formated_payload = if let Ok(x) = serde_json::to_string(&payload) { x } else {
            return Err("Invalid data was provided to the `play` json.".to_string());
        };

        {
            let mut ws = socket.lock().await;
            if let Err(why) = ws.send(TungsteniteMessage::text(formated_payload)).await {
                return Err(format!("There was an error sending the `play` payload => {:?}", why));
            };
        }

        Ok(())
    }

    pub fn replace(&mut self, replace: bool) -> &mut Self {
        self.replace = replace;
        self
    }

    pub fn start_msec(&mut self, start: u128) -> &mut Self {
        self.start = start;
        self
    }
    pub fn start_sec(&mut self, start: u128) -> &mut Self {
        self.start = start * 1000;
        self
    }

    pub fn finish_msec(&mut self, finish: u128) -> &mut Self {
        self.finish = finish;
        self
    }
    pub fn finish_sec(&mut self, finish: u128) -> &mut Self {
        self.finish = finish * 1000;
        self
    }
}

impl LavalinkClient {
    pub fn new() -> Self {
        let mut client = LavalinkClient::default();
        client.host = "localhost".to_string();
        client.port = 2333;
        client.password = "youshallnotpass".to_string();
        client.shard_count = Some(1);
        client
    }

    pub async fn initialize(&mut self) -> Result<&mut Self, TungsteniteError> {
        if self.is_ssl {
            self.socket_uri = format!("wss://{}:{}", &self.host, &self.port);
            self.rest_uri = format!("https://{}:{}", &self.host, &self.port);
        } else {
            self.socket_uri = format!("ws://{}:{}", &self.host, &self.port);
            self.rest_uri = format!("http://{}:{}", &self.host, &self.port);
        }

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", self.password.parse()?);
        headers.insert("Num-Shards", self.shard_count.unwrap_or(1).to_string().parse()?);
        headers.insert("User-Id", self.bot_id.to_string().parse()?);

        self.headers = Some(headers);

        let url = Request::builder()
            .uri(&self.socket_uri)
            .header("Authorization", &self.password)
            .header("Num-Shards", &self.shard_count.unwrap_or(1).to_string())
            .header("User-Id", &self.bot_id.to_string())
            .body(())
            .unwrap();

        let (ws_stream, _) = connect_async(url).await?;

        self.socket = Some(Arc::new(Mutex::new(ws_stream)));
        Ok(self)
    }

    pub async fn init(&mut self) -> Result<&mut Self, TungsteniteError> {
        self.initialize().await
    }

    pub async fn auto_search_tracks<TS: ToString + Display>(&self, query: TS) -> Result<Tracks, ReqwestError> {
        let r = Regex::new(r"https?://(?:www\.)?.+").unwrap();
        if r.is_match(&query.to_string()) {
            self.get_tracks(query.to_string()).await
        } else {
            self.get_tracks(format!("ytsearch:{}", query)).await
        }
    }

    pub async fn search_tracks<TS: ToString + Display>(&self, query: TS) -> Result<Tracks, ReqwestError> {
        self.get_tracks(format!("ytsearch:{}", query)).await
    }

    pub async fn get_tracks<TS: ToString+ std::convert::AsRef<str>>(&self, query: TS) -> Result<Tracks, ReqwestError> {
        let reqwest = ReqwestClient::new();
        let url = Url::parse_with_params(&format!("{}/loadtracks", &self.socket_uri), &[("identifier", &query)]).expect("The query cannot be formated to a url.");

        let resp = reqwest.get(url)
            .headers(self.headers.clone().unwrap())
            .send()
            .await?
            .json::<Tracks>()
            .await?;

        Ok(resp)
    }

    pub fn play<'a, 'b, 'c>(&'a self, handler: &'b Handler, track: &'c Track) -> PlayParameters<'a, 'b, 'c> {
        let mut p = PlayParameters::default();
        p.client = Some(self);
        p.handler = Some(handler);
        p.track = Some(track);
        p
    }
}
