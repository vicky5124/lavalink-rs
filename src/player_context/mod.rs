mod context;
mod inner;

use crate::model::*;

use std::collections::VecDeque;

pub use context::PlayerContext;
pub(crate) use inner::PlayerContextInner;

#[derive(PartialEq, Debug, Clone, Default)]
pub struct TrackInQueue {
    pub track: track::TrackData,
    pub start_time: Option<u128>,
    pub end_time: Option<u128>,
    pub volume: Option<u16>,
    pub filters: Option<player::Filters>,
}

pub(crate) enum PlayerMessage {
    UpdatePlayer(player::Player),
    UpdatePlayerTrack(Option<track::TrackData>),
    UpdatePlayerState(player::State),
    InsertToQueue(TrackInQueue),
    ReplaceQueue(VecDeque<TrackInQueue>),
    AppendQueue(VecDeque<TrackInQueue>),
    TrackFinished(bool),
    StartTrack,
    Close,
}

impl TrackInQueue {
    fn into_update_player(self) -> http::UpdatePlayer {
        http::UpdatePlayer {
            encoded_track: self.track.encoded.into(),
            position: self.start_time,
            end_time: self.end_time,
            volume: self.volume,
            filters: self.filters,
            ..Default::default()
        }
    }
}

impl From<track::TrackData> for TrackInQueue {
    fn from(track: track::TrackData) -> Self {
        Self {
            track,
            ..Default::default()
        }
    }
}

impl From<&track::TrackData> for TrackInQueue {
    fn from(track: &track::TrackData) -> Self {
        Self {
            track: track.clone(),
            ..Default::default()
        }
    }
}
