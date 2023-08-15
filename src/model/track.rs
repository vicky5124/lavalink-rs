use crate::model::deserialize_option_number;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrackLoadType {
    Track,
    Playlist,
    Search,
    Empty,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum TrackLoadData {
    Track(TrackData),
    Playlist(PlaylistData),
    Search(Vec<TrackData>),
    Error(TrackError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub load_type: TrackLoadType,
    pub data: Option<TrackLoadData>,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackData {
    pub encoded: String,
    pub info: TrackInfo,
    pub plugin_info: Option<serde_json::Value>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub identifier: String,
    pub is_seekable: bool,
    pub author: String,
    pub length: u64,
    pub is_stream: bool,
    pub position: u64,
    pub title: String,
    pub uri: Option<String>,
    pub artwork_url: Option<String>,
    pub isrc: Option<String>,
    pub source_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistData {
    pub info: PlaylistInfo,
    pub tracks: Vec<TrackData>,
    pub plugin_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    pub name: String,
    #[serde(deserialize_with = "deserialize_option_number")]
    pub selected_track: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackError {
    pub message: String,
    pub severity: String,
    pub cause: String,
}
