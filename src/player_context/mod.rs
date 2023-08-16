mod context;
mod inner;

use crate::model::*;

use std::collections::VecDeque;

pub use context::PlayerContext;
pub(crate) use inner::PlayerContextInner;

#[derive(PartialEq, Debug, Clone, Default)]
/// A track that's inside the queue.
pub struct TrackInQueue {
    /// The track itself.
    pub track: track::TrackData,
    /// The time the track should start at.
    pub start_time: Option<std::time::Duration>,
    /// The time the track should end at.
    pub end_time: Option<std::time::Duration>,
    /// The volume the track should start at.
    pub volume: Option<u16>,
    /// The filters the track should start at.
    pub filters: Option<player::Filters>,
}

pub(crate) enum PlayerMessage {
    UpdatePlayer(player::Player),
    UpdatePlayerTrack(Option<track::TrackData>),
    UpdatePlayerState(player::State),
    GetPlayer(oneshot::Sender<player::Player>),
    InsertToQueue(TrackInQueue),
    ReplaceQueue(VecDeque<TrackInQueue>),
    AppendQueue(VecDeque<TrackInQueue>),
    GetQueue(oneshot::Sender<VecDeque<TrackInQueue>>),
    TrackFinished(bool),
    StartTrack,
    Close,
}

impl TrackInQueue {
    fn into_update_player(self) -> http::UpdatePlayer {
        http::UpdatePlayer {
            encoded_track: self.track.encoded.into(),
            position: self.start_time.map(|x| x.as_millis()),
            end_time: self.end_time.map(|x| x.as_millis()),
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
