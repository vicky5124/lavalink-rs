mod context;
mod inner;

use crate::model::*;

use std::collections::VecDeque;

pub use context::PlayerContext;
pub(crate) use inner::PlayerContextInner;

#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
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
    GetPlayer(oneshot::Sender<player::Player>),
    UpdatePlayer(player::Player),
    UpdatePlayerTrack(Option<track::TrackData>),
    UpdatePlayerState(player::State),

    GetQueue(oneshot::Sender<VecDeque<TrackInQueue>>),
    SetQueue(QueueMessage),

    TrackFinished(bool),
    StartTrack,
    Close,
}

#[derive(PartialEq, Debug, Clone)]
pub enum QueueMessage {
    /// Add a track to the end of the queue.
    PushToBack(TrackInQueue),
    /// Add a track to the start of the queue.
    PushToFront(TrackInQueue),
    /// Insert a track to a specific position in the queue.
    Insert(usize, TrackInQueue),
    /// Remove a track from the queue.
    Remove(usize),
    /// Clear the queue.
    Clear,
    /// Replace the entire queue with another one.
    Replace(VecDeque<TrackInQueue>),
    /// Append a queue to the end of the current one.
    Append(VecDeque<TrackInQueue>),
}

impl TrackInQueue {
    fn into_update_player(self) -> http::UpdatePlayer {
        http::UpdatePlayer {
            encoded_track: self.track.encoded.into(),
            position: self.start_time.map(|x| x.as_millis() as u64),
            end_time: self.end_time.map(|x| x.as_millis() as u64),
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
