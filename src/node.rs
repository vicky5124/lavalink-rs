use crate::client::LavalinkClient;
use crate::error::LavalinkError;
use crate::model::{events, BoxFuture, Secret, UserId};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use arc_swap::ArcSwap;
use futures::stream::StreamExt;
use http::HeaderMap;
use http::Request;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use tokio_tungstenite::{connect_async, tungstenite::handshake::client::generate_key};

#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "python"), derive(Hash, Default))]
#[cfg_attr(feature = "python", pyo3::pyclass)]
/// A builder for the node.
///
/// # Example
///
/// ```
/// # use crate::model::UserId;
/// let node_builder = NodeBuilder {
///     hostname: "localhost:2333".to_string(),
///     password: "youshallnotpass".to_string(),
///     user_id: UserId(551759974905151548),
///     ..Default::default()
/// };
/// ```
pub struct NodeBuilder {
    /// The hostname of the Lavalink server.
    ///
    /// Example: "localhost:2333"
    pub hostname: String,
    /// If the Lavalink server is behind SSL encryption.
    pub is_ssl: bool,
    /// The event handler specific for this node.
    ///
    /// In most cases, the default is good.
    pub events: events::Events,
    /// The Lavalink server password.
    pub password: String,
    /// The bot User ID that will use Lavalink.
    pub user_id: UserId,
    /// The previous Session ID if resuming.
    pub session_id: Option<String>,
}

#[derive(Debug)]
/// A Lavalink server node.
pub struct Node {
    pub id: usize,
    pub session_id: ArcSwap<String>,
    pub websocket_address: String,
    pub http: crate::http::Http,
    pub events: events::Events,
    pub is_running: AtomicBool,
    pub(crate) password: Secret,
    pub user_id: UserId,
    pub cpu: ArcSwap<crate::model::events::Cpu>,
    pub memory: ArcSwap<crate::model::events::Memory>,
}

#[derive(Copy, Clone)]
struct EventDispatcher<'a>(&'a Node, &'a LavalinkClient);

// Thanks Alba :D
impl<'a> EventDispatcher<'a> {
    pub(crate) async fn dispatch<T, F>(self, event: T, handler: F)
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

    #[cfg(not(feature = "python"))]
    pub(crate) async fn parse_and_dispatch<T, F>(self, event: &'a str, handler: F)
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
    /// Create a connection to the Lavalink server.
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
            headers.insert("Authorization", self.password.0.parse()?);
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

        self.is_running.store(true, Ordering::SeqCst);

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
                    let ed = EventDispatcher(self_node, &lavalink_client);

                    match base_event.get("op").unwrap().as_str().unwrap() {
                        "ready" => {
                            let ready_event: events::Ready = serde_json::from_str(&x).unwrap();

                            self_node
                                .session_id
                                .swap(Arc::new(ready_event.session_id.to_string()));

                            #[cfg(feature = "python")]
                            {
                                let session_id = self_node.session_id.load_full();

                                if let Some(handler) = &self_node.events.event_handler {
                                    handler
                                        .event_ready(
                                            lavalink_client.clone(),
                                            (*session_id).clone(),
                                            ready_event.clone(),
                                        )
                                        .await;
                                }
                                if let Some(handler) = &lavalink_client.events.event_handler {
                                    handler
                                        .event_ready(
                                            lavalink_client.clone(),
                                            (*session_id).clone(),
                                            ready_event.clone(),
                                        )
                                        .await;
                                }
                            }

                            ed.dispatch(ready_event, |e| e.ready).await;
                        }
                        "playerUpdate" => {
                            let player_update_event: events::PlayerUpdate =
                                serde_json::from_str(&x).unwrap();

                            if let Some(player) =
                                lavalink_client.get_player_context(player_update_event.guild_id)
                            {
                                if let Err(why) =
                                    player.update_state(player_update_event.state.clone())
                                {
                                    error!(
                                        "Error updating state for player {}: {}",
                                        player_update_event.guild_id.0, why
                                    );
                                }
                            }

                            #[cfg(feature = "python")]
                            {
                                let session_id = self_node.session_id.load_full();

                                if let Some(handler) = &self_node.events.event_handler {
                                    handler
                                        .event_player_update(
                                            lavalink_client.clone(),
                                            (*session_id).clone(),
                                            player_update_event.clone(),
                                        )
                                        .await;
                                }
                                if let Some(handler) = &lavalink_client.events.event_handler {
                                    handler
                                        .event_player_update(
                                            lavalink_client.clone(),
                                            (*session_id).clone(),
                                            player_update_event.clone(),
                                        )
                                        .await;
                                }
                            }

                            ed.dispatch(player_update_event, |e| e.player_update).await;
                        }
                        "stats" => {
                            #[cfg(feature = "python")]
                            {
                                let event: events::Stats = serde_json::from_str(&x).unwrap();
                                let session_id = self_node.session_id.load_full();

                                self_node.cpu.store(Arc::new(event.cpu.clone()));
                                self_node.memory.store(Arc::new(event.memory.clone()));

                                if let Some(handler) = &self_node.events.event_handler {
                                    handler
                                        .event_stats(
                                            lavalink_client.clone(),
                                            (*session_id).clone(),
                                            event.clone(),
                                        )
                                        .await;
                                }
                                if let Some(handler) = &lavalink_client.events.event_handler {
                                    handler
                                        .event_stats(
                                            lavalink_client.clone(),
                                            (*session_id).clone(),
                                            event.clone(),
                                        )
                                        .await;
                                }

                                ed.dispatch(event, |e| e.stats).await;
                            }
                            #[cfg(not(feature = "python"))]
                            ed.parse_and_dispatch(&x, |e| e.stats).await;
                        }
                        "event" => match base_event.get("type").unwrap().as_str().unwrap() {
                            "TrackStartEvent" => {
                                let track_event: events::TrackStart =
                                    serde_json::from_str(&x).unwrap();

                                if let Some(player) =
                                    lavalink_client.get_player_context(track_event.guild_id)
                                {
                                    if let Err(why) =
                                        player.update_track(track_event.track.clone().into())
                                    {
                                        error!(
                                            "Error sending update track message for player {}: {}",
                                            track_event.guild_id.0, why
                                        );
                                    }
                                }

                                #[cfg(feature = "python")]
                                {
                                    let session_id = self_node.session_id.load_full();

                                    if let Some(handler) = &self_node.events.event_handler {
                                        handler
                                            .event_track_start(
                                                lavalink_client.clone(),
                                                (*session_id).clone(),
                                                track_event.clone(),
                                            )
                                            .await;
                                    }
                                    if let Some(handler) = &lavalink_client.events.event_handler {
                                        handler
                                            .event_track_start(
                                                lavalink_client.clone(),
                                                (*session_id).clone(),
                                                track_event.clone(),
                                            )
                                            .await;
                                    }
                                }

                                ed.dispatch(track_event, |e| e.track_start).await;
                            }
                            "TrackEndEvent" => {
                                let track_event: events::TrackEnd =
                                    serde_json::from_str(&x).unwrap();

                                if let Some(player) =
                                    lavalink_client.get_player_context(track_event.guild_id)
                                {
                                    if let Err(why) =
                                        player.finish(track_event.reason.clone().into())
                                    {
                                        error!(
                                            "Error sending finish message for player {}: {}",
                                            track_event.guild_id.0, why
                                        );
                                    }

                                    if let Err(why) =
                                        player.update_track(track_event.track.clone().into())
                                    {
                                        error!(
                                            "Error sending update track message for player {}: {}",
                                            track_event.guild_id.0, why
                                        );
                                    }
                                }

                                #[cfg(feature = "python")]
                                {
                                    let session_id = self_node.session_id.load_full();

                                    if let Some(handler) = &self_node.events.event_handler {
                                        handler
                                            .event_track_end(
                                                lavalink_client.clone(),
                                                (*session_id).clone(),
                                                track_event.clone(),
                                            )
                                            .await;
                                    }
                                    if let Some(handler) = &lavalink_client.events.event_handler {
                                        handler
                                            .event_track_end(
                                                lavalink_client.clone(),
                                                (*session_id).clone(),
                                                track_event.clone(),
                                            )
                                            .await;
                                    }
                                }

                                ed.dispatch(track_event, |e| e.track_end).await;
                            }
                            "TrackExceptionEvent" => {
                                #[cfg(feature = "python")]
                                {
                                    let event: events::TrackException =
                                        serde_json::from_str(&x).unwrap();
                                    let session_id = self_node.session_id.load_full();

                                    if let Some(handler) = &self_node.events.event_handler {
                                        handler
                                            .event_track_exception(
                                                lavalink_client.clone(),
                                                (*session_id).clone(),
                                                event.clone(),
                                            )
                                            .await;
                                    }
                                    if let Some(handler) = &lavalink_client.events.event_handler {
                                        handler
                                            .event_track_exception(
                                                lavalink_client.clone(),
                                                (*session_id).clone(),
                                                event.clone(),
                                            )
                                            .await;
                                    }

                                    ed.dispatch(event, |e| e.track_exception).await;
                                }
                                #[cfg(not(feature = "python"))]
                                ed.parse_and_dispatch(&x, |e| e.track_exception).await;
                            }
                            "TrackStuckEvent" => {
                                #[cfg(feature = "python")]
                                {
                                    let event: events::TrackStuck =
                                        serde_json::from_str(&x).unwrap();
                                    let session_id = self_node.session_id.load_full();

                                    if let Some(handler) = &self_node.events.event_handler {
                                        handler
                                            .event_track_stuck(
                                                lavalink_client.clone(),
                                                (*session_id).clone(),
                                                event.clone(),
                                            )
                                            .await;
                                    }
                                    if let Some(handler) = &lavalink_client.events.event_handler {
                                        handler
                                            .event_track_stuck(
                                                lavalink_client.clone(),
                                                (*session_id).clone(),
                                                event.clone(),
                                            )
                                            .await;
                                    }

                                    ed.dispatch(event, |e| e.track_stuck).await;
                                }
                                #[cfg(not(feature = "python"))]
                                ed.parse_and_dispatch(&x, |e| e.track_stuck).await;
                            }
                            "WebSocketClosedEvent" => {
                                #[cfg(feature = "python")]
                                {
                                    let event: events::WebSocketClosed =
                                        serde_json::from_str(&x).unwrap();
                                    let session_id = self_node.session_id.load_full();

                                    if let Some(handler) = &self_node.events.event_handler {
                                        handler
                                            .event_websocket_closed(
                                                lavalink_client.clone(),
                                                (*session_id).clone(),
                                                event.clone(),
                                            )
                                            .await;
                                    }
                                    if let Some(handler) = &lavalink_client.events.event_handler {
                                        handler
                                            .event_websocket_closed(
                                                lavalink_client.clone(),
                                                (*session_id).clone(),
                                                event.clone(),
                                            )
                                            .await;
                                    }

                                    ed.dispatch(event, |e| e.websocket_closed).await;
                                }
                                #[cfg(not(feature = "python"))]
                                ed.parse_and_dispatch(&x, |e| e.websocket_closed).await;
                            }
                            _ => (),
                        },

                        _ => (),
                    }

                    ed.dispatch(base_event, |e| e.raw).await;
                });
            }

            let self_node = lavalink_client.nodes.get(self_node_id).unwrap();
            self_node.is_running.store(false, Ordering::SeqCst);
            error!("Connection Closed.");
        });

        Ok(())
    }
}
