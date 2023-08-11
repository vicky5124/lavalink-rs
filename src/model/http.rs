use crate::model::*;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoded_track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<player::Filters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<player::ConnectionInfo>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ResumingState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resuming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub version: Version,
    pub build_time: u64,
    pub git: Git,
    pub jvm: String,
    pub lavaplayer: String,
    pub source_managers: Vec<String>,
    pub filters: Vec<String>,
    pub plugins: Vec<Plugin>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Git {
    pub branch: String,
    pub commit: String,
    pub commit_time: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub semver: String,
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub pre_release: Option<String>,
    pub build: Option<String>,
}
