use crate::error::LavalinkError;
use crate::WsStream;

use serenity::model::id::GuildId as SerenityGuildId;

use serde_json::{
    json,
    Value,
};
use serde_aux::prelude::*;
use serde::{
    Deserialize,
    Serialize
};

use futures::{
    sink::SinkExt,
    stream::{
        SplitSink,
    },
};
use async_tungstenite::{
    tungstenite::Message as TungsteniteMessage,
};


pub type LavalinkResult<T> = Result<T, LavalinkError>;

fn merge(a: &mut Value, b: Value) {
    match (a, b) {
        (a @ &mut Value::Object(_), Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                merge(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

// thanks twilight for this :P
/// The type of event that something is.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub enum SendOpcode {
    /// Destroy a player from a node.
    Destroy,
    // /// Equalize a player.
    // Equalizer,
    /// Pause a player.
    Pause(Pause),
    /// Play a track.
    Play(Play),
    /// Seek a player's active track to a new position.
    Seek(Seek),
    /// Stop a player.
    Stop,
    /// A combined voice server and voice state update.
    VoiceUpdate(VoiceUpdate),
    /// Set the volume of a player.
    Volume(Volume),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Play {
    pub track: String,
    pub no_replace: bool,
    pub start_time: u64,
    pub end_time: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceUpdate {
    pub session_id: String,
    pub event: Event
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub token: String,
    pub endpoint: String,
    pub guild_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub volume: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Seek {
    pub position: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pause {
    pub pause: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct GuildId(pub u64);

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

impl SendOpcode {
    pub async fn send(&self, guild_id: impl Into<GuildId>, socket: &mut SplitSink<WsStream, TungsteniteMessage>) -> LavalinkResult<()> {
        let value = match self {
            Self::Destroy => {
                json!({
                    "op" : self,
                    "guildId" : &guild_id.into().0.to_string()
                })
            },
            Self::Stop => {
                json!({
                    "op" : self,
                    "guildId" : &guild_id.into().0.to_string()
                })
            },
            Self::Seek(data) => {
                let mut x = json!({
                    "op" : "seek",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            },
            Self::Pause(data) => {
                let mut x = json!({
                    "op" : "pause",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            },
            Self::Play(data) => {
                let mut x = json!({
                    "op" : "play",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            },
            Self::VoiceUpdate(data) => {
                let mut x = json!({
                    "op" : "voiceUpdate",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            },
            Self::Volume(data) => {
                let mut x = json!({
                    "op" : "volume",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            },
        };

        let payload = serde_json::to_string(&value).unwrap();

        {
            if let Err(why) = socket.send(TungsteniteMessage::text(&payload)).await {
                return Err(LavalinkError::ErrorSendingVoiceUpdatePayload(why));
            };
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    pub guild: GuildId,

    pub now_playing: Option<TrackQueue>,
    pub is_paused: bool,
    pub volume: u16,
    pub queue: Vec<TrackQueue>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TrackQueue {
    pub track: Track,
    pub start_time: u64,
    pub end_time: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Tracks {
    #[serde(rename = "playlistInfo")]
    pub playlist_info: PlaylistInfo,

    #[serde(rename = "loadType")]
    pub load_type: String,

    pub tracks: Vec<Track>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct PlaylistInfo {
    #[serde(rename = "selectedTrack")]
    pub selected_track: Option<i64>,

    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Track {
    pub track: String,
    pub info: Option<Info>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Info {
    #[serde(rename = "isSeekable")]
    pub is_seekable: bool,

    #[serde(rename = "isStream")]
    pub is_stream: bool,

    pub identifier: String,
    pub author: String,
    pub length: u64,
    pub position: u64,
    pub title: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawEvent {
    #[serde(rename = "playingPlayers")]
    pub playing_players: Option<i64>,
    pub op: String,
    pub memory: Option<Memory>,
    #[serde(rename = "frameStats")]
    pub frame_stats: Option<FrameStats>,
    pub players: Option<i64>,
    pub cpu: Option<Cpu>,
    pub uptime: Option<i64>,
    pub state: Option<State>,
    #[serde(rename = "guildId")]
    pub guild_id: Option<String>,
    #[serde(rename = "type")]
    pub raw_event_type: Option<String>,
    pub track: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cpu {
    pub cores: i64,
    #[serde(rename = "systemLoad")]
    pub system_load: f64,
    #[serde(rename = "lavalinkLoad")]
    pub lavalink_load: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrameStats {
    pub sent: i64,
    pub deficit: i64,
    pub nulled: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Memory {
    pub reservable: i64,
    pub used: i64,
    pub free: i64,
    pub allocated: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct State {
    pub position: i64,
    pub time: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GatewayEvent {
    pub op: String,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stats {
    #[serde(rename = "playingPlayers")]
    pub playing_players: i64,
    pub op: String,
    pub memory: Memory,
    #[serde(rename = "frameStats")]
    pub frame_stats: Option<FrameStats>,
    pub players: i64,
    pub cpu: Cpu,
    pub uptime: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerUpdate {
    pub op: String,
    pub state: State,
    #[serde(rename = "guildId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackStart {
    pub op: String,
    #[serde(rename = "type")]
    pub track_start_type: String,
    pub track: String,
    #[serde(rename = "guildId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackFinish {
    pub op: String,
    pub reason: String,
    #[serde(rename = "type")]
    pub track_finish_type: String,
    pub track: String,
    #[serde(rename = "guildId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: u64,
}
