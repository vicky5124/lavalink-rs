use crate::gateway::LavalinkEventHandler;
use crate::model::*;
#[cfg(feature = "discord-gateway")]
use crate::voice::{raw_handle_event_voice_server_update, raw_handle_event_voice_state_update};
use crate::LavalinkClient;
use crate::WsStream;

#[cfg(feature = "discord-gateway")]
use async_tungstenite::tokio::connect_async;
use futures::stream::{SplitStream, StreamExt};
#[cfg(feature = "discord-gateway")]
use futures::SinkExt;
#[cfg(feature = "discord-gateway")]
use http::Request;
#[cfg(feature = "discord-gateway")]
use serde::Deserialize;
#[cfg(feature = "discord-gateway")]
use serde_json::json;
#[cfg(feature = "discord-gateway")]
use std::sync::Arc;
#[cfg(feature = "discord-gateway")]
use std::time::Duration;
#[cfg(feature = "discord-gateway")]
use tokio::sync::mpsc;
#[cfg(feature = "discord-gateway")]
use tokio::sync::RwLock;

use async_tungstenite::tungstenite::Message as TungsteniteMessage;

#[cfg(feature = "discord-gateway")]
#[derive(Deserialize)]
struct HeartBeatInner {
    heartbeat_interval: u64,
}

#[cfg(feature = "discord-gateway")]
#[derive(Debug, Deserialize)]
struct BaseEvent<T> {
    d: T,
    //t: Option<String>,
}

#[cfg(feature = "discord-gateway")]
#[derive(Debug, Deserialize)]
struct BaseEventNoData {
    t: Option<String>,
    s: Option<usize>,
}

#[cfg(feature = "discord-gateway")]
pub async fn discord_event_loop(client: LavalinkClient, token: &str, mut wait_time: Duration) {
    let reconnect = Arc::new(RwLock::new(false));
    let was_reconnected = Arc::new(RwLock::new(false));
    let session_id = Arc::new(RwLock::new(String::new()));
    let seq = Arc::new(RwLock::new(0_usize));
    let rec_seq = Arc::new(RwLock::new(0_usize));

    loop {
        let headers = client
            .discord_gateway_data()
            .await
            .lock()
            .await
            .headers
            .clone();
        let socket_uri = client
            .discord_gateway_data()
            .await
            .lock()
            .await
            .socket_uri
            .clone();

        let mut url_builder = Request::builder();

        {
            let ref_headers = url_builder.headers_mut().unwrap();
            *ref_headers = headers.clone();
        }

        let url = url_builder.uri(socket_uri).body(()).unwrap();

        let (ws_stream, _) = connect_async(url).await.unwrap();

        let (mut write, mut read) = ws_stream.split();

        debug!("Waiting before connecting to the discord websocket.");

        // wait before starting to not get rate limited.
        tokio::time::sleep(wait_time).await;
        wait_time = Duration::from_secs(0);

        debug!("Connecting to the discord websocket.");

        let discord_ws = client.discord_gateway_data().await;
        let (tx, mut rx) = mpsc::unbounded_channel();

        discord_ws.lock().await.sender = tx.clone();

        let first = read.next().await;

        let tx_hb = tx.clone();
        let was_reconnected_clone = was_reconnected.clone();

        match first {
            Some(Ok(v)) => {
                let heartbeat: BaseEvent<HeartBeatInner> =
                    serde_json::from_str(&v.into_text().unwrap()).unwrap();

                tokio::spawn(async move {
                    let mut val = 1_usize;
                    loop {
                        tokio::time::sleep(Duration::from_millis(heartbeat.d.heartbeat_interval))
                            .await;

                        if *was_reconnected_clone.read().await {
                            *was_reconnected_clone.write().await = false;
                            break;
                        } else {
                            // thread 'tokio-runtime-worker' panicked at 'called `Result::unwrap()` on an `Err` value: SendError("{\"op\":1,\"d\":64}")', /home/nitsuga/.cargo/git/checkouts/lavalink-rs-38e41c1b59bb345b/0900b34/src/event_loops.rs:108:78
                            // note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

                            let _ = tx_hb.send(format!(r#"{{"op":1,"d":{}}}"#, val));
                            val += 1;
                        }
                    }
                });
            }
            Some(Err(why)) => panic!("Failed to connect to the discord gateway: {}", why),
            None => panic!("Failed to connect to the discord gateway: No Reason Provided"),
        }

        let identify = if !*reconnect.read().await {
            json!({
                "op": 2,
                "d": {
                    //"compress": true, // implement this when i figure out how to deserialize binary
                    "large_threshold": 250,
                    "token": token,
                    "intents": 1 << 7, // GUILD_VOICE_STATES // 128
                    "v": "v9",
                    "properties": {
                        "$browser": "lavalink-rs",
                        "$device": "lavalink-rs",
                        "$os": std::env::consts::OS,
                    },
                },
            })
        } else {
            *reconnect.write().await = false;
            *was_reconnected.write().await = true;
            let session_id = session_id.read().await.clone();
            let seq = seq.read().await.clone();
            let rec_seq_inner = rec_seq.read().await.clone();
            warn!(
                "Session: {}, Seq: {}, Last recon Seq: {}",
                session_id, seq, rec_seq_inner
            );

            if seq == rec_seq_inner {
                let tx_hb = tx.clone();
                let _ = tx_hb.send("reconnect".to_string());
                break;
            }

            *rec_seq.write().await = seq;

            json!({
                "op": 6,
                "d": {
                  "token": token,
                  "session_id": &session_id,
                  "seq": seq
                }
            })
        };

        let payload = serde_json::to_string(&identify).unwrap();

        let identify_request = write.send(TungsteniteMessage::text(payload)).await;

        debug!("identify_request: {:#?}", identify_request);

        let tx_hb = tx.clone();
        let client_clone = client.clone();
        let session_id_clone = session_id.clone();
        let seq_clone = seq.clone();
        let reconnect_clone = reconnect.clone();

        tokio::spawn(async move {
            'events: while let Some(Ok(resp)) = read.next().await {
                if *reconnect_clone.read().await == true {
                    break 'events;
                }
                debug!("event: {:#?}", resp);

                let text_resp = if resp.is_close() {
                    info!("Close event obtained: {}", resp);
                    let resp_text = resp.to_string();
                    if resp_text.starts_with("Discord") {
                        *reconnect_clone.write().await = true;
                        tx_hb.send("reconnect".to_string()).unwrap();
                        continue 'events;
                    } else {
                        tx_hb.send("reconnect".to_string()).unwrap();
                        break 'events;
                    }
                } else if let Ok(x) = resp.clone().into_text() {
                    x
                } else {
                    warn!("Other event type obtained: {}", resp);
                    continue 'events;
                };

                //let event: BaseEvent<String> = serde_json::from_str(&text_resp).unwrap();
                let event_name: BaseEventNoData = serde_json::from_str(&text_resp).unwrap();

                if let Some(s) = event_name.s {
                    *seq_clone.write().await = s;
                }

                match event_name.t.unwrap_or_default().as_str() {
                    "READY" => {
                        let event: BaseEvent<EventReady> =
                            serde_json::from_str(&text_resp).unwrap();

                        *session_id_clone.write().await = event.d.session_id;
                        info!("Lavalink discord gateway ready event received.");
                    }
                    "VOICE_STATE_UPDATE" => {
                        let event: BaseEvent<EventVoiceStateUpdate> =
                            serde_json::from_str(&text_resp).unwrap();
                        debug!("Voice State Update");
                        debug!("{:#?}", event);

                        raw_handle_event_voice_state_update(
                            &client_clone,
                            event.d.guild_id,
                            event.d.channel_id,
                            event.d.user_id,
                            event.d.session_id,
                        )
                        .await;
                    }
                    "VOICE_SERVER_UPDATE" => {
                        let event: BaseEvent<EventVoiceServerUpdate> =
                            serde_json::from_str(&text_resp).unwrap();
                        debug!("Voice Server Update");
                        debug!("{:#?}", event);

                        raw_handle_event_voice_server_update(
                            &client_clone,
                            event.d.guild_id,
                            event.d.endpoint,
                            event.d.token,
                        )
                        .await;
                    }
                    "RESUMED" => info!("Resumed the discord websocket."),
                    "" => (),
                    _ => debug!("Unknown event: {}", &text_resp),
                }
            }

            // Guarentee reconnect
            *reconnect_clone.write().await = true;
            let _ = tx_hb.send("reconnect".to_string());

            warn!("Stopped getting events.");
        });

        while let Some(v) = rx.recv().await {
            if &v == "reconnect" {
                break;
            }

            if let Err(why) = write.send(TungsteniteMessage::text(v)).await {
                error!("Error sending discord event: {}", why);
            }
        }
    }
}

pub async fn lavalink_event_loop(
    mut read: SplitStream<WsStream>,
    handler: impl LavalinkEventHandler + Send + Sync + 'static,
    client: LavalinkClient,
) {
    while let Some(Ok(resp)) = read.next().await {
        if let TungsteniteMessage::Text(x) = &resp {
            if let Ok(base_event) = serde_json::from_str::<GatewayEvent>(x) {
                match base_event.op.as_str() {
                    "stats" => {
                        if let Ok(stats) = serde_json::from_str::<Stats>(x) {
                            handler.stats(client.clone(), stats).await;
                        }
                    }
                    "playerUpdate" => {
                        if let Ok(player_update) = serde_json::from_str::<PlayerUpdate>(x) {
                            {
                                let client_clone = client.clone();
                                let client_lock = client_clone.inner.lock().await;

                                if let Some(mut node) =
                                    client_lock.nodes.get_mut(&player_update.guild_id.0)
                                {
                                    if let Some(mut current_track) = node.now_playing.as_mut() {
                                        let mut info =
                                            current_track.track.info.as_mut().unwrap().clone();
                                        info.position = player_update.state.position as u64;
                                        current_track.track.info = Some(info);
                                        trace!(
                                            "Updated track {:?} with position {}",
                                            current_track.track.info.as_ref().unwrap(),
                                            player_update.state.position
                                        );
                                    }
                                };
                            }

                            handler.player_update(client.clone(), player_update).await;
                        }
                    }
                    "event" => match base_event.event_type.unwrap().as_str() {
                        "WebSocketClosedEvent" => {
                            if let Ok(websocket_closed) = serde_json::from_str::<WebSocketClosed>(x)
                            {
                                handler
                                    .websocket_closed(client.clone(), websocket_closed)
                                    .await;
                            }
                        }
                        "PlayerDestroyedEvent" => {
                            if let Ok(player_destroyed) = serde_json::from_str::<PlayerDestroyed>(x)
                            {
                                handler
                                    .player_destroyed(client.clone(), player_destroyed)
                                    .await;
                            }
                        }
                        "TrackStartEvent" => {
                            if let Ok(track_start) = serde_json::from_str::<TrackStart>(x) {
                                handler.track_start(client.clone(), track_start).await;
                            }
                        }
                        "TrackEndEvent" => {
                            if let Ok(track_finish) = serde_json::from_str::<TrackFinish>(x) {
                                if track_finish.reason == "FINISHED" {
                                    let client_lock = client.inner.lock().await;

                                    if let Some(mut node) =
                                        client_lock.nodes.get_mut(&track_finish.guild_id.0)
                                    {
                                        if !node.queue.is_empty() {
                                            node.queue.remove(0);
                                        }
                                        node.now_playing = None;
                                    };
                                }

                                handler.track_finish(client.clone(), track_finish).await;
                            }
                        }
                        "TrackExceptionEvent" => {
                            if let Ok(track_exception) = serde_json::from_str::<TrackException>(x) {
                                handler
                                    .track_exception(client.clone(), track_exception)
                                    .await;
                            }
                        }
                        "TrackStuckEvent" => {
                            if let Ok(track_stuck) = serde_json::from_str::<TrackStuck>(x) {
                                handler.track_stuck(client.clone(), track_stuck).await;
                            }
                        }
                        _ => warn!("Unknown event: {}", &x),
                    },
                    _ => warn!("Unknown socket response: {}", &x),
                }
            }
        }
    }
}
