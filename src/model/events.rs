use crate::client::LavalinkClient;
use crate::model::*;

#[derive(Hash, Debug, Clone, Default)]
pub struct Events {
    pub raw: Option<fn(LavalinkClient, session_id: String, &serde_json::Value) -> BoxFuture<()>>,
    pub ready: Option<fn(LavalinkClient, session_id: String, &Ready) -> BoxFuture<()>>,
    pub player_update:
        Option<fn(LavalinkClient, session_id: String, &PlayerUpdate) -> BoxFuture<()>>,
    pub stats: Option<fn(LavalinkClient, session_id: String, &Stats) -> BoxFuture<()>>,
    pub track_start: Option<fn(LavalinkClient, session_id: String, &TrackStart) -> BoxFuture<()>>,
    pub track_end: Option<fn(LavalinkClient, session_id: String, &TrackEnd) -> BoxFuture<()>>,
    pub track_exception:
        Option<fn(LavalinkClient, session_id: String, &TrackException) -> BoxFuture<()>>,
    pub track_stuck: Option<fn(LavalinkClient, session_id: String, &TrackStuck) -> BoxFuture<()>>,
    pub websocket_closed:
        Option<fn(LavalinkClient, session_id: String, &WebSocketClosed) -> BoxFuture<()>>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    pub op: String,
    pub session_id: String,
    pub resumed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerUpdate {
    pub op: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    pub state: player::State,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    #[serde(default)]
    pub op: String,
    pub players: u64,
    pub playing_players: u64,
    pub uptime: u64,
    pub memory: Memory,
    pub cpu: Cpu,
    pub frame_stats: Option<FrameStats>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cpu {
    pub cores: u64,
    pub system_load: f64,
    pub lavalink_load: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameStats {
    pub sent: u64,
    pub nulled: u64,
    pub deficit: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    pub free: u64,
    pub used: u64,
    pub allocated: u64,
    pub reservable: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackStart {
    pub op: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    pub track: track::TrackData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackEnd {
    pub op: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    pub track: track::TrackData,
    pub reason: TrackEndReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrackEndReason {
    Finished,
    LoadFailed,
    Stopped,
    Replaced,
    Cleanup,
}

impl From<TrackEndReason> for bool {
    fn from(value: TrackEndReason) -> Self {
        match value {
            TrackEndReason::Finished => true,
            TrackEndReason::LoadFailed => true,
            TrackEndReason::Stopped => false,
            TrackEndReason::Replaced => false,
            TrackEndReason::Cleanup => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackException {
    pub op: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    pub track: track::TrackData,
    pub exception: track::TrackError,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackStuck {
    pub op: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    pub track: track::TrackData,
    pub threshold_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketClosed {
    pub op: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    pub code: u16,
    pub reason: String,
    pub by_remote: bool,
}
