use crate::model::*;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// Information about the player of a guild.
pub struct Player {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub guild_id: GuildId,
    /// The currently playing track.
    pub track: Option<track::TrackData>,
    /// The current volume of the player.
    pub volume: u16,
    /// Whether the player is paused or not.
    pub paused: bool,
    /// The state of the player.
    pub state: State,
    /// The filters currently in use by the player
    pub filters: Option<Filters>,
    /// The voice connection information of the player.
    pub voice: ConnectionInfo,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
pub struct State {
    /// Unix timestamp in milliseconds.
    pub time: u64,
    /// The current position of the track in milliseconds.
    pub position: u64,
    /// Whether Lavalink is connected to the discord voice gateway.
    pub connected: bool,
    #[serde(deserialize_with = "deserialize_option_number")]
    /// The latency of the node to the Discord voice gateway in milliseconds.
    ///
    /// None if not connected.
    pub ping: Option<u32>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// Discord voice websocket connection information.
pub struct ConnectionInfo {
    /// The Discord voice endpoint to connect to.
    ///
    /// Provided by `Voice Server Update`.
    pub endpoint: String,
    /// The Discord voice token to authenticate with.
    ///
    /// Provided by `Voice Server Update`.
    pub token: String,
    /// The Discord voice session id to authenticate with.
    ///
    /// Not to be confused by the Lavalink `session_id`.
    ///
    /// Provided by `Voice State Update`.
    pub session_id: String,
}

impl ConnectionInfo {
    pub fn fix(&mut self) {
        self.endpoint = self.endpoint.replace("wss://", "");
    }
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

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass)]
pub struct Filters {
    /// Adjusts the player volume from 0.0 to 5.0, where 1.0 is 100%.
    ///
    /// NOTE: Values >1.0 may cause clipping
    pub volume: Option<u16>,
    /// Adjusts 15 different bands.
    pub equalizer: Option<Vec<Equalizer>>,
    /// Eliminates part of a band, usually targeting vocals.
    pub karaoke: Option<Karaoke>,
    /// Changes the speed, pitch, and rate.
    pub timescale: Option<Timescale>,
    /// Creates a shuddering effect, where the volume quickly oscillates.
    pub tremolo: Option<TremoloVibrato>,
    /// Creates a shuddering effect, where the pitch quickly oscillates.
    pub vibrato: Option<TremoloVibrato>,
    /// Rotates the audio around the stereo channels/user headphones (aka Audio Panning).
    pub rotation: Option<Rotation>,
    /// Distorts the audio.
    pub distortion: Option<Distortion>,
    /// Mixes both stereo channels (left and right).
    pub channel_mix: Option<ChannelMix>,
    /// Filters out higher frequencies.
    pub low_pass: Option<LowPass>,
    /// Filter plugin configurations.
    pub plugin_filters: Option<serde_json::Value>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// Mixes both channels (left and right), with a configurable factor on how much each channel affects the other.
///
/// With the defaults, both channels are kept independent of each other.
/// Setting all factors to 0.5 means both channels get the same audio.
/// All values are (0.0 <= x <= 1.0)
pub struct ChannelMix {
    pub left_to_left: Option<f64>,
    pub left_to_right: Option<f64>,
    pub right_to_left: Option<f64>,
    pub right_to_right: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// Distortion effect.
///
/// It can generate some pretty unique audio effects.
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

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// A fixed band equalizer.
pub struct Equalizer {
    /// The band (0 to 14)
    pub band: u8,
    /// The gain (-0.25 to 1.0)
    ///
    /// -0.25 means the given band is completely muted, and 0.25 means it is doubled.
    /// Modifying the gain could also change the volume of the output.
    pub gain: f64,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// Uses equalization to eliminate part of a band, usually targeting vocals.
pub struct Karaoke {
    /// The level (0 to 1.0 where 0.0 is no effect and 1.0 is full effect)
    pub level: Option<f64>,
    /// The mono level (0 to 1.0 where 0.0 is no effect and 1.0 is full effect)
    pub mono_level: Option<f64>,
    /// The filter band (in Hz)
    pub filter_band: Option<f64>,
    /// The filter width.
    pub filter_width: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// Higher frequencies get suppressed, while lower frequencies pass through this filter.
pub struct LowPass {
    /// The smoothing factor (1.0 < x)
    ///
    /// Any smoothing values equal to or less than 1.0 will disable the filter.
    pub smoothing: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// Rotates the sound around the stereo channels/user headphones (aka Audio Panning).
///
/// It can produce an effect similar to [this](https://youtu.be/QB9EB8mTKcc) without the reverb.
pub struct Rotation {
    /// The frequency of the audio rotating around the listener in Hz.
    ///
    /// 0.2 is similar to the example video above
    pub rotation_hz: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// Changes the speed, pitch, and rate.
///
/// All default to 1.0.
pub struct Timescale {
    /// The playback speed (0.0 <= x)
    pub speed: Option<f64>,
    /// The pitch (0.0 <= x)
    pub pitch: Option<f64>,
    /// The rate (0.0 <= x)
    pub rate: Option<f64>,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// Tremolo uses amplification to create a shuddering effect, where the volume quickly oscillates.
///
/// [Demo](https://en.wikipedia.org/wiki/File:Fuse_Electronics_Tremolo_MK-III_Quick_Demo.ogv)
///
/// Vibrato is similar to tremolo, but rather than oscillating the volume, it oscillates the pitch.
pub struct TremoloVibrato {
    /// For tremolo (0.0 < x)
    /// For vibrato (0.0 < x <= 14.0)
    pub frequency: Option<f64>,
    /// For both tremolo and vibrato (0.0 < x <= 1.0)
    pub depth: Option<f64>,
}
