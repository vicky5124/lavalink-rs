// oh god, this module looks terrible

use crate::error::LavalinkResult;
use crate::WsStream;

use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;
use std::sync::Arc;

use typemap_rev::TypeMap;

#[cfg(feature = "serenity")]
use serenity_dep::model::id::{
    ChannelId as SerenityChannelId, GuildId as SerenityGuildId, UserId as SerenityUserId,
};

#[cfg(feature = "twilight")]
use twilight_model::id::{
    ChannelId as TwilightChannelId, GuildId as TwilightGuildId, UserId as TwilightUserId,
};

#[cfg(feature = "songbird")]
use songbird_dep::id::{
    ChannelId as SongbirdChannelId, GuildId as SongbirdGuildId, UserId as SongbirdUserId,
};

use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use serde_json::{json, Value};

use futures::{sink::SinkExt, stream::SplitSink};

use async_tungstenite::tungstenite::Message as TungsteniteMessage;
use tokio::sync::RwLock;

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
    pub event: Event,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub token: String,
    pub endpoint: String,
    #[serde(deserialize_with = "deserialize_string_from_number")]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub u64);

#[cfg(feature = "serenity")]
impl From<SerenityGuildId> for GuildId {
    fn from(guild_id: SerenityGuildId) -> GuildId {
        GuildId(guild_id.0)
    }
}

#[cfg(feature = "twilight")]
impl From<TwilightGuildId> for GuildId {
    fn from(guild_id: TwilightGuildId) -> GuildId {
        GuildId(guild_id.0)
    }
}

#[cfg(feature = "songbird")]
impl From<SongbirdGuildId> for GuildId {
    fn from(guild_id: SongbirdGuildId) -> GuildId {
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

#[cfg(feature = "serenity")]
impl From<SerenityUserId> for UserId {
    fn from(user_id: SerenityUserId) -> UserId {
        UserId(user_id.0)
    }
}

#[cfg(feature = "twilight")]
impl From<TwilightUserId> for UserId {
    fn from(user_id: TwilightUserId) -> UserId {
        UserId(user_id.0)
    }
}

#[cfg(feature = "songbird")]
impl From<SongbirdUserId> for UserId {
    fn from(user_id: SongbirdUserId) -> UserId {
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

#[cfg(feature = "serenity")]
impl From<SerenityChannelId> for ChannelId {
    fn from(user_id: SerenityChannelId) -> ChannelId {
        ChannelId(user_id.0)
    }
}

#[cfg(feature = "twilight")]
impl From<TwilightChannelId> for ChannelId {
    fn from(user_id: TwilightChannelId) -> ChannelId {
        ChannelId(user_id.0)
    }
}

#[cfg(feature = "songbird")]
impl From<SongbirdChannelId> for ChannelId {
    fn from(user_id: SongbirdChannelId) -> ChannelId {
        ChannelId(user_id.0)
    }
}

impl From<u64> for ChannelId {
    fn from(user_id: u64) -> ChannelId {
        ChannelId(user_id)
    }
}

impl From<i64> for ChannelId {
    fn from(user_id: i64) -> ChannelId {
        ChannelId(user_id as u64)
    }
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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

impl FromStr for ChannelId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(u64::from_str(s)?))
    }
}

impl FromStr for UserId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(u64::from_str(s)?))
    }
}

impl FromStr for GuildId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(u64::from_str(s)?))
    }
}

impl GuildId {
    #[inline]
    #[cfg(feature = "serenity")]
    pub fn to_serenity(&self) -> SerenityGuildId {
        SerenityGuildId(self.0)
    }

    #[inline]
    #[cfg(feature = "twilight")]
    pub fn to_twilight(&self) -> TwilightGuildId {
        TwilightGuildId(self.0)
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
    #[cfg(feature = "serenity")]
    pub fn to_serenity(&self) -> SerenityUserId {
        SerenityUserId(self.0)
    }

    #[inline]
    #[cfg(feature = "twilight")]
    pub fn to_twilight(&self) -> TwilightUserId {
        TwilightUserId(self.0)
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

impl ChannelId {
    #[inline]
    #[cfg(feature = "serenity")]
    pub fn to_serenity(&self) -> SerenityChannelId {
        SerenityChannelId(self.0)
    }

    #[inline]
    #[cfg(feature = "twilight")]
    pub fn to_twilight(&self) -> TwilightChannelId {
        TwilightChannelId(self.0)
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
    pub async fn send(
        &self,
        guild_id: impl Into<GuildId>,
        socket: &mut SplitSink<WsStream, TungsteniteMessage>,
    ) -> LavalinkResult<()> {
        let value = match self {
            Self::Destroy => {
                json!({
                    "op" : self,
                    "guildId" : &guild_id.into().0.to_string()
                })
            }
            Self::Stop => {
                json!({
                    "op" : self,
                    "guildId" : &guild_id.into().0.to_string()
                })
            }
            Self::Seek(data) => {
                let mut x = json!({
                    "op" : "seek",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            }
            Self::Pause(data) => {
                let mut x = json!({
                    "op" : "pause",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            }
            Self::Play(data) => {
                let mut x = json!({
                    "op" : "play",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            }
            Self::VoiceUpdate(data) => {
                let mut x = json!({
                    "op" : "voiceUpdate",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            }
            Self::Volume(data) => {
                let mut x = json!({
                    "op" : "volume",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            }
            Self::Equalizer(data) => {
                let mut x = json!({
                    "op" : "equalizer",
                    "guildId" : &guild_id.into().0.to_string(),
                });
                merge(&mut x, serde_json::to_value(data).unwrap());
                x
            }
        };

        let payload = serde_json::to_string(&value).unwrap();

        {
            if let Err(why) = socket.send(TungsteniteMessage::text(&payload)).await {
                return Err(why.into());
            };
        }

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub guild: GuildId,

    pub now_playing: Option<TrackQueue>,
    pub is_paused: bool,
    pub volume: u16,
    pub queue: Vec<TrackQueue>,
    /// Check used to know if the loop is on Client.loops
    pub is_on_loops: bool,
    /// Use this to store whatever information you wish that's guild specific, such as invocation
    /// channel id's, for example.
    #[serde(skip)]
    pub data: Arc<RwLock<TypeMap>>,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            guild: GuildId(0),
            now_playing: None,
            is_paused: false,
            volume: 100,
            queue: vec![],
            is_on_loops: false,
            data: Arc::new(RwLock::new(TypeMap::new())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct TrackQueue {
    pub track: Track,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub requester: Option<UserId>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Tracks {
    #[serde(rename = "playlistInfo")]
    pub playlist_info: Option<PlaylistInfo>,

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
    #[serde(default)]
    pub is_seekable: bool,

    #[serde(rename = "isStream")]
    #[serde(default)]
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
    pub guild_id: GuildId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackStart {
    pub op: String,
    #[serde(rename = "type")]
    pub track_start_type: String,
    pub track: String,
    #[serde(rename = "guildId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSocketClosed {
    pub op: String,
    #[serde(rename = "type")]
    pub websocket_closed_type: String,
    #[serde(rename = "userId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub user_id: UserId,
    #[serde(rename = "guildId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    pub code: u64,
    #[serde(rename = "byRemote")]
    pub by_remote: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerDestroyed {
    pub op: String,
    #[serde(rename = "type")]
    pub player_destroyed_type: String,
    pub cleanup: bool,
    #[serde(rename = "guildId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    #[serde(rename = "userId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub user_id: UserId,
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
    pub guild_id: GuildId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackException {
    pub exception: Exception,
    pub op: String,
    #[serde(rename = "type")]
    pub track_exception_type: String,
    pub track: String,
    pub error: String,
    #[serde(rename = "guildId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Exception {
    pub severity: String,
    pub cause: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackStuck {
    pub op: String,
    #[serde(rename = "thresholdMs")]
    pub threshold_ms: u64,
    #[serde(rename = "type")]
    pub track_stuck_type: String,
    pub track: String,
    #[serde(rename = "guildId")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
}

#[cfg(feature = "discord-gateway")]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConnectionInfo {
    pub guild_id: Option<GuildId>,
    pub channel_id: Option<ChannelId>,
    pub endpoint: Option<String>,
    pub token: Option<String>,
    pub session_id: Option<String>,
}

#[cfg(feature = "discord-gateway")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventVoiceServerUpdate {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    pub endpoint: String,
    pub token: String,
}

#[cfg(feature = "discord-gateway")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventVoiceStateUpdate {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    pub channel_id: Option<ChannelId>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub user_id: UserId,
    pub session_id: String,
}

#[cfg(feature = "discord-gateway")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventReady {
    pub session_id: String,
}
