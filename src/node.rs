use crate::client::LavalinkClient;
use crate::error::LavalinkError;
use crate::model::{events, BoxFuture, UserId};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use arc_swap::ArcSwap;
use async_tungstenite::tungstenite::Message as TungsteniteMessage;
use async_tungstenite::{tokio::connect_async, tungstenite::handshake::client::generate_key};
use futures::stream::StreamExt;
use http::Request;
use reqwest::header::HeaderMap;

#[derive(Hash, Debug, Clone)]
pub struct NodeBuilder {
    pub hostname: String,
    pub is_ssl: bool,
    pub events: events::Events,
    pub password: String,
    pub user_id: UserId,
    pub session_id: Option<String>,
}

#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub session_id: ArcSwap<String>,
    pub websocket_address: String,
    pub http: crate::http::Http,
    pub events: events::Events,
    pub is_running: AtomicBool,
    pub password: String,
    pub user_id: UserId,
}

#[derive(Copy, Clone)]
struct EventDispatcher<'a>(&'a Node, &'a LavalinkClient);

// Thanks Alba :D
impl<'a> EventDispatcher<'a> {
    pub async fn dispatch<T, F>(self, event: T, handler: F)
    where
        F: Fn(&events::Events) -> Option<fn(LavalinkClient, String, &T) -> BoxFuture<()>>,
    {
        let EventDispatcher(self_node, lavalink_client) = self;
        let session_id = self_node.session_id.load_full();
        let targets = [&self_node.events, &lavalink_client.events].into_iter();

        for handler in targets.filter_map(handler) {
            handler(lavalink_client.clone(), (*session_id).clone(), &event).await;
        }
    }

    pub async fn parse_and_dispatch<T, F>(self, event: &'a str, handler: F)
    where
        F: Fn(&events::Events) -> Option<fn(LavalinkClient, String, &T) -> BoxFuture<()>>,
        T: serde::Deserialize<'a>,
    {
        trace!("{:?}", event);
        let event = serde_json::from_str(event).unwrap();
        self.dispatch(event, handler).await
    }
}

impl Node {
    pub async fn connect(&self, lavalink_client: LavalinkClient) -> Result<(), LavalinkError> {
        let mut url = Request::builder()
            .method("GET")
            .header("Host", &self.websocket_address)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .uri(&self.websocket_address)
            .body(())?;

        {
            let ref_headers = url.headers_mut();

            let mut headers = HeaderMap::new();
            headers.insert("Authorization", self.password.parse()?);
            headers.insert("User-Id", self.user_id.0.to_string().parse()?);
            headers.insert("Session-Id", self.session_id.to_string().parse()?);
            headers.insert(
                "Client-Name",
                format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"),)
                    .to_string()
                    .parse()?,
            );

            ref_headers.extend(headers.clone());
        }

        let (ws_stream, _) = connect_async(url).await?;

        info!("Connected to {}", self.websocket_address);

        let (_write, mut read) = ws_stream.split();

        self.is_running.store(true, Ordering::Relaxed);

        let self_node_id = self.id;

        tokio::spawn(async move {
            while let Some(Ok(resp)) = read.next().await {
                let x = match resp {
                    TungsteniteMessage::Text(x) => x,
                    _ => continue,
                };

                let base_event = match serde_json::from_str::<serde_json::Value>(&x) {
                    Ok(base_event) => base_event,
                    _ => continue,
                };

                let lavalink_client = lavalink_client.clone();

                tokio::spawn(async move {
                    let self_node = lavalink_client.nodes.get(self_node_id).unwrap();
                    let ed = EventDispatcher(&self_node, &lavalink_client);

                    match base_event.get("op").unwrap().as_str().unwrap() {
                        "ready" => {
                            let ready_event: events::Ready = serde_json::from_str(&x).unwrap();

                            self_node
                                .session_id
                                .swap(Arc::new(ready_event.session_id.to_string()));

                            ed.dispatch(ready_event, |e| e.ready).await
                        }
                        "playerUpdate" => ed.parse_and_dispatch(&x, |e| e.player_update).await,
                        "stats" => ed.parse_and_dispatch(&x, |e| e.stats).await,
                        "event" => match base_event.get("type").unwrap().as_str().unwrap() {
                            "TrackStartEvent" => ed.parse_and_dispatch(&x, |e| e.track_start).await,
                            "TrackEndEvent" => ed.parse_and_dispatch(&x, |e| e.track_end).await,
                            "TrackExceptionEvent" => {
                                ed.parse_and_dispatch(&x, |e| e.track_exception).await
                            }
                            "TrackStuckEvent" => ed.parse_and_dispatch(&x, |e| e.track_stuck).await,
                            "WebSocketClosedEvent" => {
                                ed.parse_and_dispatch(&x, |e| e.websocket_closed).await
                            }
                            _ => (),
                        },

                        _ => (),
                    }

                    ed.dispatch(base_event, |e| e.raw).await
                });
            }

            let self_node = lavalink_client.nodes.get(self_node_id).unwrap();
            self_node.is_running.store(false, Ordering::Relaxed);
            error!("Connection Closed.");
        });

        Ok(())
    }
}
