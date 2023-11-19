use crate::model::deserialize_option_number;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// The type of data returned when loading a track.
pub enum TrackLoadType {
    Track,
    Playlist,
    Search,
    Empty,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
/// The data returned when loading a track.
pub enum TrackLoadData {
    /// A track has been loaded.
    Track(TrackData),
    /// A playlist has been loaded.
    Playlist(PlaylistData),
    /// A search result has been loaded.
    Search(Vec<TrackData>),
    /// Loading has failed with an error.
    Error(TrackError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Loaded track data.
pub struct Track {
    /// The type of the result.
    pub load_type: TrackLoadType,
    /// The data of the result.
    pub data: Option<TrackLoadData>,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass)]
/// Information about a track.
pub struct TrackData {
    /// The base64 encoded track data.
    pub encoded: String,
    /// Info and metadata about the track.
    pub info: TrackInfo,
    /// Addition track info provided by plugins.
    pub plugin_info: Option<serde_json::Value>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
/// Information and metadata about the track.
pub struct TrackInfo {
    /// The track identifier.
    pub identifier: String,
    /// Whether the track is seekable.
    pub is_seekable: bool,
    /// The track author.
    pub author: String,
    /// The track length in milliseconds.
    pub length: u64,
    /// Whether the track is a stream.
    pub is_stream: bool,
    /// The track starting position in milliseconds.
    pub position: u64,
    /// The track title,
    pub title: String,
    /// The track uri.
    pub uri: Option<String>,
    /// The track artwork url.
    pub artwork_url: Option<String>,
    /// The track "International Standard Recording Code".
    pub isrc: Option<String>,
    /// The track source name.
    pub source_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass)]
pub struct PlaylistData {
    /// The information of the playlist.
    pub info: PlaylistInfo,
    /// The tracks of the playlist.
    pub tracks: Vec<TrackData>,
    /// Addition playlist information provided by plugins.
    pub plugin_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
pub struct PlaylistInfo {
    /// The name of the playlist.
    pub name: String,
    #[serde(deserialize_with = "deserialize_option_number")]
    /// The selected track of the playlist.
    ///
    /// None if no track is selected.
    pub selected_track: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
pub struct TrackError {
    /// The message of the exception.
    pub message: String,
    /// The severity of the exception.
    pub severity: String,
    /// The cause of the exception.
    pub cause: String,
}
