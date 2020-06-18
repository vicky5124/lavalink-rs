//! DO NOT USE THIS
//! It's very unstable and purely in development, and it's a really bad implementation.

use crate::{
    WebsocketConnection,
    error::LavalinkError,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use serenity::model::id::GuildId as SerenityGuildId;

use serde_json::json;
use serde::Deserialize;

use tokio::sync::Mutex;
use futures::prelude::*;
use http::Request;
//use reqwest::{
//    Client as ReqwestClient,
//    header::*,
//    Url,
//};
use async_tungstenite::{
    tungstenite::Message as TungsteniteMessage,
    tokio::connect_async,
};

static SOCKET_URI: &str = "wss://gateway.discord.gg/?v=6&encoding=json";
static REST_URI: &str = "https://discord.com/api/v6/";

#[derive(Debug, Deserialize)]
struct SocketResponse {
    op: i64,
    d: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    heartbeat_interval: i64,
}

#[derive(Debug, Copy, Clone)]
pub struct GuildId(u64);

#[derive(Clone)]
pub struct GatewayRest {
    discord_token: String,
    socket: WebsocketConnection,
    pub handlers: HashMap<GuildId, Handler>,
}

#[derive(Debug, Clone)]
pub struct Handler {
    token: String,
    endpoint: String,
    session_id: String,
}

impl From<SerenityGuildId> for GuildId {
    fn from(guild_id: SerenityGuildId) -> GuildId {
        GuildId(guild_id.0)
    }
}

impl From<u64> for GuildId {
    fn from(guild_id: u64) -> GuildId {
        GuildId(guild_id)
    }
}

impl From<i64> for GuildId {
    fn from(guild_id: i64) -> GuildId {
        GuildId(guild_id as u64)
    }
}

async fn start_socket(token: String) -> Result<GatewayRest, LavalinkError> {
    let socket_url = Request::builder()
        .uri(SOCKET_URI)
        .header("Authorization", &format!("Bot {}", token))
        .header("bot", "True")
        .header("Content-type", "application/json")
        .body(())
        .unwrap();

    //let rest_url = Request::builder()
    //    .uri(REST_URI)
    //    .header("Authorization", &format!("Bot {}", token))
    //    .header("bot", "True")
    //    .header("Content-type", "application/json")
    //    .body(())
    //    .unwrap();

    let (ws_stream, _) = connect_async(socket_url).await.unwrap();
    let socket = Arc::new(Mutex::new(ws_stream));
    let ws_clone = Arc::clone(&socket);
    let ws_clone2 = Arc::clone(&socket);

    {
        let payload = json!({
            "op": 2,
            "d": {
                "token": &token,
                "properties": {
                    "$os": "linux",
                    "$browser": "serenity-lavalink",
                    "$device": "serenity-lavalink"
                }
            }
        });

        let formated_payload = serde_json::to_string(&payload).unwrap();

        let mut ws = socket.lock().await;
        ws.send(TungsteniteMessage::text(&formated_payload)).await.unwrap();
        if let Some(x) = ws.next().await {
            let msg = x.unwrap();
            if let TungsteniteMessage::Text(m) = msg {
                let data = serde_json::from_str::<SocketResponse>(&m).unwrap();
                let heartbeat = data.d.heartbeat_interval;


                tokio::spawn(async move {
                    let mut x = 1;
                    loop {
                        let mut payload_heartbeat: HashMap<&str, usize> = HashMap::new();
                        payload_heartbeat.insert("op", 1);
                        payload_heartbeat.insert("d", x);
                        let formated_payload_heartbeat = serde_json::to_string(&payload).unwrap();

                        let mut socket = ws_clone.lock().await;
                        socket.send(TungsteniteMessage::text(&formated_payload_heartbeat)).await.unwrap();
                        println!("Sent heartbeat.");
                        drop(socket);
                        x += 1;
                        tokio::time::delay_for(Duration::from_millis(heartbeat as u64)).await;
                    }
                });

                tokio::spawn(async move {
                    loop {
                        let mut socket = ws_clone2.lock().await;
                        while let Some(x) = socket.next().await {
                            dbg!(&x);
                        }
                        drop(socket);

                        tokio::time::delay_for(Duration::from_millis(1000)).await;
                    }
                });
            }
        }
    }

    Ok(GatewayRest {
        discord_token: token,
        handlers: HashMap::new(),
        socket: Arc::clone(&socket),
    })
}

impl GatewayRest {
    pub async fn start(discord_token: impl ToString) -> Result<Self, LavalinkError> {
        let token = discord_token.to_string();
        start_socket(token).await
    }

}
