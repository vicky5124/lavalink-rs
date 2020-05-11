use crate::{
    Track,
    LavalinkClient,
};

use std::{
    sync::{
        Arc,
        //RwLock,
    },
    time::Duration,
    cmp::{
        min,
        max,
    },
};

use serenity::{
    model::id::{
        GuildId,
        ChannelId,
    },
    voice::Handler,
};

use serde_json::json;

use tokio::sync::RwLock;
use futures::prelude::*;
use async_tungstenite::tungstenite::Message as TungsteniteMessage;

#[derive(Clone, Debug, Default)]
pub struct Node {
    pub now_playing: Option<TrackQueue>,
    pub now_playing_time_left: Option<Duration>,
    pub now_playing_start_time: Option<Duration>,
    pub is_paused: bool,
    pub queue: Vec<TrackQueue>,
    pub channel: ChannelId,
    pub guild: GuildId,
    pub volume: u16,
}

#[derive(Clone, Debug)]
pub struct TrackQueue {
    pub track: Track,
    pub start: u64,
    pub finish: u64,
}

#[derive(Default)]
pub struct NodePlay<'a, 'b> {
    pub client: Option<&'a LavalinkClient>,
    pub replace: bool,
    pub track: Option<Track>,
    pub start: u64,
    pub finish: u64,
    pub node: Option<&'b mut Node>,
}

impl<'a, 'b> NodePlay<'a, 'b> {
    pub fn queue(self) {
        if let Some(node) = self.node {
            let track = if self.finish != 0 {
                TrackQueue {
                    track: self.track.as_ref().unwrap().clone(),
                    start: self.start,
                    finish: self.finish,
                }
            } else {
                TrackQueue {
                    track: self.track.clone().unwrap(),
                    start: self.start,
                    finish: self.track.as_ref().unwrap().info.length as u64,
                }
            };

            if self.replace {
                node.queue.reverse();
                node.queue.push(track);
                node.queue.reverse();
                node.now_playing = None;
                node.now_playing_start_time = None;
                node.now_playing_time_left = None;
            } else {
                node.queue.push(track);
            }
        }
    }

    pub fn replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }

    pub fn start_time(mut self, start: Duration) -> Self {
        self.start = start.as_millis() as u64;
        self
    }

    pub fn finish_time(mut self, finish: Duration) -> Self {
        self.finish = finish.as_millis() as u64;
        self
    }
}

impl Node {
    pub fn new(lava_client: &mut LavalinkClient, guild_id: GuildId, channel_id: ChannelId) -> &mut Self {
        let mut node = Node::default();
        node.guild =  guild_id;
        node.channel = channel_id; 
        lava_client.nodes.insert(guild_id, node);
        lava_client.nodes.get_mut(&guild_id).unwrap()
    }

    pub async fn stop(&mut self, lava_client: &mut LavalinkClient, guild_id: &GuildId) -> Result<(), String> {
        let socket = if let Some(x) = &lava_client.socket { x } else {
            return Err("There is no initialized websocket.".to_string());
        };

        let payload = json!({
            "op" : "stop",
            "guildId" : guild_id.0.to_string()
        });

        let formated_payload = if let Ok(x) = serde_json::to_string(&payload) { x } else {
            return Err("Invalid data was provided to the `stop` json.".to_string());
        };

        {
            let mut ws = socket.lock().await;
            if let Err(why) = ws.send(TungsteniteMessage::text(formated_payload)).await {
                return Err(format!("There was an error sending the `stop` operation => {:?}", why));
            };
        }

        self.now_playing = None;
        self.now_playing_time_left = None;
        self.queue = Vec::new();

        Ok(())
    }

    pub fn skip(&mut self) {
        self.now_playing_time_left = None;
    }

    pub async fn destroy(&self, lava_client: &mut LavalinkClient, guild_id: &GuildId) -> Result<(), String> {
        let socket = if let Some(x) = &lava_client.socket { x } else {
            return Err("There is no initialized websocket.".to_string());
        };

        let payload = json!({
            "op" : "destroy",
            "guildId" : guild_id.0.to_string()
        });

        let formated_payload = if let Ok(x) = serde_json::to_string(&payload) { x } else {
            return Err("Invalid data was provided to the `destroy` json.".to_string());
        };

        {
            let mut ws = socket.lock().await;
            if let Err(why) = ws.send(TungsteniteMessage::text(formated_payload)).await {
                return Err(format!("There was an error sending the `destroy` operation => {:?}", why));
            };
        }

        for (index, guild) in lava_client.loops.clone().iter().enumerate() {
            if guild == guild_id {
                lava_client.loops.remove(index);
            }
        }
        lava_client.nodes.remove(&guild_id);

        Ok(())
    }

    pub fn play<'b>(&'b mut self, track: Track) -> NodePlay {
        let mut p = NodePlay::default();
        p.track = Some(track);
        p.node = Some(self);
        p
    }

    pub async fn set_pause(&mut self, lava_client: &LavalinkClient, guild_id: &GuildId, pause: bool) -> Result<(), String> {
        let socket = if let Some(x) = &lava_client.socket { x } else {
            return Err("There is no initialized websocket.".to_string());
        };

        let payload = json!({"op" : "pause",
            "guildId" : guild_id.0.to_string(),
            "pause" : pause
        });

        let formated_payload = if let Ok(x) = serde_json::to_string(&payload) { x } else {
            return Err("Invalid data was provided to the `pause` json.".to_string());
        };

        {
            let mut ws = socket.lock().await;
            if let Err(why) = ws.send(TungsteniteMessage::text(formated_payload)).await {
                return Err(format!("There was an error sending the `pause` status => {:?}", why));
            };
        }

        self.is_paused = pause;

        Ok(())
    }

    pub async fn pause(&mut self, lava_client: &LavalinkClient, guild_id: &GuildId) -> Result<(), String> {
        self.set_pause(lava_client, guild_id, true).await
    }
    pub async fn resume(&mut self, lava_client: &LavalinkClient, guild_id: &GuildId) -> Result<(), String> {
        self.set_pause(lava_client, guild_id, false).await
    }

    pub async fn set_volume(&mut self, lava_client: &LavalinkClient, guild_id: &GuildId, mut volume: u16) -> Result<(), String> {
        volume = max(min(volume, 1000), 0);
        let socket = if let Some(x) = &lava_client.socket { x } else {
            return Err("There is no initialized websocket.".to_string());
        };

        let payload = json!({"op" : "volume",
            "guildId" : guild_id.0.to_string(),
            "volume" : volume
        });

        let formated_payload = if let Ok(x) = serde_json::to_string(&payload) { x } else {
            return Err("Invalid data was provided to the `volume` json.".to_string());
        };

        {
            let mut ws = socket.lock().await;
            if let Err(why) = ws.send(TungsteniteMessage::text(formated_payload)).await {
                return Err(format!("There was an error sending the `volume` value => {:?}", why));
            };
        }

        self.volume = volume;

        Ok(())
    }

    pub async fn jump_to_time(&mut self, lava_client: &LavalinkClient, guild_id: &GuildId, time: Duration) -> Result<(), String> {
        let socket = if let Some(x) = &lava_client.socket { x } else {
            return Err("There is no initialized websocket.".to_string());
        };

        let payload = json!({"op" : "seek",
            "guildId" : guild_id.0.to_string(),
            "position" : time.as_millis().to_string()
        });

        let formated_payload = if let Ok(x) = serde_json::to_string(&payload) { x } else {
            return Err("Invalid data was provided to the `seek` json.".to_string());
        };

        {
            let mut ws = socket.lock().await;
            if let Err(why) = ws.send(TungsteniteMessage::text(formated_payload)).await {
                return Err(format!("There was an error sending the `seek` timestamp => {:?}", why));
            };
        }

        self.now_playing_start_time = Some(time);
        if self.now_playing_time_left.is_some() {
            self.now_playing_time_left = Some(Duration::from_millis(self.now_playing.as_ref().unwrap().finish));
            self.now_playing_time_left = self.now_playing_time_left.unwrap().checked_sub(time + Duration::from_secs(2));
        }

        Ok(())
    }

    pub async fn scrub(&mut self, lava_client: &LavalinkClient, guild_id: &GuildId, time: Duration) -> Result<(), String> {
        self.jump_to_time(lava_client, guild_id, time).await
    }
    pub async fn seek(&mut self, lava_client: &LavalinkClient, guild_id: &GuildId, time: Duration) -> Result<(), String> {
        self.jump_to_time(lava_client, guild_id, time).await
    }

    pub async fn start_loop(&self, lava_client: Arc<RwLock<LavalinkClient>>, handler: Arc<Handler>) {
        let lava_clone = Arc::clone(&lava_client);
        let handler_clone = Arc::clone(&handler);
        let guild = self.guild;

        tokio::spawn(async move {
            let socket = {
                let read_lock = lava_clone.read().await;
                if let Some(x) = &read_lock.socket { x.clone() } else {
                    panic!("There is no initialized websocket.");
                }
            };
            let guild_id = handler_clone.guild_id.0.to_string();

            let token = if let Some(x) = handler_clone.token.as_ref() { x } else {
                panic!("No `token` was found on the handler.");
            };
            let endpoint = if let Some(x) = handler_clone.endpoint.as_ref() { x } else {
                panic!("No `endpoint` was found on the handler.");
            };

            let event = json!({
                "token" : &token,
                "guild_id" : &guild_id,
                "endpoint" : &endpoint
            });

            let session_id = if let Some(x) = handler_clone.session_id.as_ref() { x } else {
                panic!("No `session id` was found on the handler.");
            };

            let payload = json!({
                "op" : "voiceUpdate",
                "guildId" : &guild_id,
                "sessionId" : &session_id,
                "event" : event
            });

            let formated_payload = if let Ok(x) = serde_json::to_string(&payload) { x } else {
                panic!("Invalid data was provided to the `voiceUpdate` json.");
            };

            {
                let mut ws = socket.lock().await;
                if let Err(why) = ws.send(TungsteniteMessage::text(formated_payload)).await {
                    panic!(format!("There was an error sending the `voiceUpdate` payload => {:?}", why));
                };
            }

            {  
                let mut lava_write = lava_clone.write().await;
                lava_write.loops.push(guild);
            }
            loop {
                {
                    let lava_write = lava_clone.read().await;
                    if !lava_write.loops.contains(&guild) {
                        break;
                    }
                }

                let mut lava_write = lava_clone.write().await;
                let node = lava_write.nodes.get_mut(&guild).unwrap();
                tokio::time::delay_for(Duration::from_secs(1)).await;
                if !node.is_paused {
                    if let Some(x) = node.now_playing_time_left {
                        node.now_playing_time_left = x.checked_sub(Duration::from_secs(1))
                    }
                }

                if node.now_playing_time_left.is_none() {
                    if !node.queue.is_empty() {
                        let track = &node.queue[0];
                        node.now_playing_time_left = Some(Duration::from_millis(track.finish));
                        let payload = {
                            if track.finish > 0 {
                                json!({
                                    "op" : "play",
                                    "guildId" : &guild_id,
                                    "track" : track.track.track,
                                    "noReplace" : false,
                                    "startTime" : track.start,
                                    "endTime" : track.finish
                                })
                            } else {
                                json!({
                                    "op" : "play",
                                    "guildId" : &guild_id,
                                    "track" : track.track.track,
                                    "noReplace" : false,
                                    "startTime" : track.start,
                                })
                            }
                        };

                        let formated_payload = if let Ok(x) = serde_json::to_string(&payload) { x } else {
                            panic!("Invalid data was provided to the `play` json.");
                        };

                        {
                            let mut ws = socket.lock().await;
                            if let Err(why) = ws.send(TungsteniteMessage::text(formated_payload)).await {
                                panic!(format!("There was an error sending the `play` payload => {:?}", why));
                            };
                        }
                        node.now_playing = Some(track.clone());
                        node.queue.remove(0);
                    } else {
                        node.now_playing = None;
                    }
                }
            }
        });
    }
}
