use crate::model::*;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    pub track: Option<track::TrackData>,
    pub volume: u16,
    pub paused: bool,
    pub state: State,
    pub voice: ConnectionInfo,
    pub filters: Option<Filters>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub time: u64,
    pub position: u64,
    pub connected: bool,
    #[serde(deserialize_with = "deserialize_option_number")]
    pub ping: Option<u32>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub endpoint: String,
    pub token: String,
    pub session_id: String,
}

#[cfg(feature = "songbird")]
use songbird_dep::ConnectionInfo as SongbirdConnectionInfo;

#[cfg(feature = "songbird")]
impl From<SongbirdConnectionInfo> for ConnectionInfo {
    fn from(connection_info: SongbirdConnectionInfo) -> ConnectionInfo {
        ConnectionInfo {
            endpoint: connection_info.endpoint,
            token: connection_info.token,
            session_id: connection_info.session_id,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    pub volume: Option<u16>,
    pub equalizer: Option<Vec<Equalizer>>,
    pub karaoke: Option<Karaoke>,
    pub timescale: Option<Timescale>,
    pub tremolo: Option<Tremolo>,
    pub vibrato: Option<Tremolo>,
    pub rotation: Option<Rotation>,
    pub distortion: Option<Distortion>,
    pub channel_mix: Option<ChannelMix>,
    pub low_pass: Option<LowPass>,
    pub plugin_filters: Option<serde_json::Value>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMix {
    pub left_to_left: Option<f64>,
    pub left_to_right: Option<f64>,
    pub right_to_left: Option<f64>,
    pub right_to_right: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Distortion {
    pub sin_offset: Option<f64>,
    pub sin_scale: Option<f64>,
    pub cos_offset: Option<f64>,
    pub cos_scale: Option<f64>,
    pub tan_offset: Option<f64>,
    pub tan_scale: Option<f64>,
    pub offset: Option<f64>,
    pub scale: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Equalizer {
    pub band: u8,
    pub gain: f64,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Karaoke {
    pub level: Option<f64>,
    pub mono_level: Option<f64>,
    pub filter_band: Option<f64>,
    pub filter_width: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LowPass {
    pub smoothing: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rotation {
    pub rotation_hz: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timescale {
    pub speed: Option<f64>,
    pub pitch: Option<f64>,
    pub rate: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tremolo {
    pub frequency: Option<f64>,
    pub depth: Option<f64>,
}
