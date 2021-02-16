use crate::error::LavalinkError;
use crate::WsStream;

use std::fmt;
use std::sync::Arc;

use typemap_rev::TypeMap;

use serenity::model::id::{
    GuildId as SerenityGuildId,
    UserId as SerenityUserId,
};

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
    stream::SplitSink,
};

#[cfg(feature = "tokio-02-marker")]
use tokio_compat as tokio;

#[cfg(feature = "tokio-02-marker")]
use async_tungstenite_compat as async_tungstenite;

use async_tungstenite::tungstenite::Message as TungsteniteMessage;
use tokio::sync::RwLock;


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
    /// Equalize a player.
    Equalizer(Equalizer),
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
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Equalizer {
    pub bands: Vec<Band>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Band {
    pub band: u8,
    pub gain: f64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub struct GuildId(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub struct UserId(pub u64);

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

impl From<SerenityUserId> for UserId {
    fn from(user_id: SerenityUserId) -> UserId {
        UserId(user_id.0)
    }
}

impl From<u64> for UserId {
    fn from(user_id: u64) -> UserId {
        UserId(user_id)
    }
}

impl From<i64> for UserId {
    fn from(user_id: i64) -> UserId {
        UserId(user_id as u64)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)  
    }
}

impl fmt::Display for GuildId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl GuildId {
    #[inline]
    pub fn to_serenity(&self) -> SerenityGuildId {
        SerenityGuildId(self.0)
    }

    #[inline]
    pub fn as_u64(&self) -> &u64 {
        &self.0
    }

    #[inline]
    pub fn as_mut_u64(&mut self) -> &mut u64 {
        &mut self.0
    }
}

impl UserId {
    #[inline]
    pub fn to_serenity(&self) -> SerenityUserId {
        SerenityUserId(self.0)
    }

    #[inline]
    pub fn as_u64(&self) -> &u64 {
        &self.0
    }

    #[inline]
    pub fn as_mut_u64(&mut self) -> &mut u64 {
        &mut self.0
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
            Self::Equalizer(data) => {
                let mut x = json!({
                    "op" : "equalizer",
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

#[derive(Clone)]
pub struct Node {
    pub guild: GuildId,

    pub now_playing: Option<TrackQueue>,
    pub is_paused: bool,
    pub volume: u16,
    pub queue: Vec<TrackQueue>,
    /// Use this to store whatever information you wish that's guild specific, such as invocation
    /// channel id's, for example.
    pub data: Arc<RwLock<TypeMap>> ,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            guild: GuildId(0),
            now_playing: None,
            is_paused: false,
            volume: 100,
            queue: vec![],
            data: Arc::new(RwLock::new(TypeMap::new())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TrackQueue {
    pub track: Track,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub requester: Option<UserId>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Tracks {
    #[serde(rename = "playlistInfo")]
    #[cfg(feature = "andesite")]
    pub playlist_info: Option<PlaylistInfo>,
    #[cfg(not(feature = "andesite"))]
    pub playlist_info: PlaylistInfo,

    /// This field will be an Enum in the future, but for now only `LOAD_FAILED` and `TRACK_LOADED`
    /// are known if you need to handle them.
    #[serde(rename = "loadType")]
    pub load_type: String,

    #[serde(default = "Vec::new")]
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
