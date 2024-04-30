use crate::client::LavalinkClient;
use crate::error::{LavalinkError, LavalinkResult};
use crate::model::*;

use std::collections::VecDeque;
//use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use ::http::Method;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
/// The player context.
pub struct PlayerContext {
    pub guild_id: GuildId,
    pub client: LavalinkClient,
    pub(crate) tx: UnboundedSender<super::PlayerMessage>,
    pub(crate) user_data: std::sync::Arc<dyn std::any::Any + Send + Sync>,
}

#[derive(Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
/// A reference to the player queue
pub struct QueueRef {
    pub(crate) tx: UnboundedSender<super::PlayerMessage>,
    pub(crate) stream:
        std::sync::Arc<std::sync::Mutex<dyn futures::Stream<Item = super::TrackInQueue> + Send>>,
}

impl PlayerContext {
    /// Close the current player.
    pub fn close(self) -> LavalinkResult<()> {
        self.tx.send(super::PlayerMessage::Close)?;
        Ok(())
    }

    /// Skip the current track and play the next in the queue.
    pub fn skip(&self) -> LavalinkResult<()> {
        self.tx.send(super::PlayerMessage::StartTrack)?;
        Ok(())
    }

    /// Finish the current track.
    ///
    /// # Parameters
    ///
    /// - `should_continue`: if the next track in the queue should play.
    pub fn finish(&self, should_continue: bool) -> LavalinkResult<()> {
        self.tx
            .send(super::PlayerMessage::TrackFinished(should_continue))?;
        Ok(())
    }

    /// Update player data in the context.
    pub fn update_player_data(&self, player: player::Player) -> LavalinkResult<()> {
        self.tx.send(super::PlayerMessage::UpdatePlayer(player))?;
        Ok(())
    }

    pub(crate) fn update_track(&self, track: Option<track::TrackData>) -> LavalinkResult<()> {
        self.tx
            .send(super::PlayerMessage::UpdatePlayerTrack(track))?;
        Ok(())
    }

    pub(crate) fn update_state(&self, state: player::State) -> LavalinkResult<()> {
        self.tx
            .send(super::PlayerMessage::UpdatePlayerState(state))?;
        Ok(())
    }

    /// Add a track to the end of the queue.
    pub fn queue(&self, track: impl Into<super::TrackInQueue>) -> LavalinkResult<()> {
        let q = self.get_queue();
        q.send(super::QueueMessage::PushToBack(track.into()))
    }

    /// Get a reference to the current queue.
    pub fn get_queue(&self) -> QueueRef {
        let stream = futures::stream::unfold((0, self.tx.clone()), |(idx, outer_tx)| async move {
            let (tx, rx) = oneshot::channel();

            let _ = outer_tx.send(super::PlayerMessage::QueueMessage(
                super::QueueMessage::GetTrack(idx, tx),
            ));

            rx.await
                .ok()
                .flatten()
                .map(|track| (track, (idx + 1, outer_tx)))
        });

        QueueRef {
            tx: self.tx.clone(),
            stream: std::sync::Arc::new(std::sync::Mutex::new(stream)),
        }
    }

    /// Get the current player information.
    pub async fn get_player(&self) -> LavalinkResult<player::Player> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(super::PlayerMessage::GetPlayer(tx))?;

        Ok(rx.await?)
    }

    /// Request a raw player update.
    pub async fn update_player(
        &self,
        update_player: &http::UpdatePlayer,
        no_replace: bool,
    ) -> LavalinkResult<player::Player> {
        let node = self.client.get_node_for_guild(self.guild_id).await;

        let result = node
            .http
            .update_player(
                self.guild_id,
                &node.session_id.load(),
                update_player,
                no_replace,
            )
            .await?;

        self.tx
            .send(super::PlayerMessage::UpdatePlayer(result.clone()))?;

        Ok(result)
    }

    /// Try and play a track. Does not change tracks if one is already playing.
    ///
    /// NOTE: Does not modify the queue.
    pub async fn play(&self, track: &track::TrackData) -> LavalinkResult<player::Player> {
        self.update_player(
            &http::UpdatePlayer {
                track: Some(http::UpdatePlayerTrack {
                    encoded: Some(track.encoded.to_string()),
                    user_data: track.user_data.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            true,
        )
        .await
    }

    /// Force play a track, replacing the current track.
    ///
    /// NOTE: Does not modify the queue.
    pub async fn play_now(&self, track: &track::TrackData) -> LavalinkResult<player::Player> {
        self.update_player(
            &http::UpdatePlayer {
                track: Some(http::UpdatePlayerTrack {
                    encoded: Some(track.encoded.to_string()),
                    user_data: track.user_data.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            false,
        )
        .await
    }

    /// Stop playing the current track.
    ///
    /// This does not continue playback of the queue.
    pub async fn stop_now(&self) -> LavalinkResult<player::Player> {
        let node = self.client.get_node_for_guild(self.guild_id).await;

        let path = node.http.path_to_uri(
            &format!(
                "/sessions/{}/players/{}",
                &node.session_id.load(),
                self.guild_id.0
            ),
            true,
        )?;

        let result = node
            .http
            .request(
                Method::PATCH,
                path,
                Some(&serde_json::json!({"track" : {"encoded": null}})),
            )
            .await?;

        let player = serde_json::from_value::<crate::error::RequestResult<player::Player>>(result)?
            .to_result()?;

        self.tx
            .send(super::PlayerMessage::UpdatePlayer(player.clone()))?;

        Ok(player)
    }

    /// Set the pause state of the player.
    pub async fn set_pause(&self, pause: bool) -> LavalinkResult<player::Player> {
        self.update_player(
            &http::UpdatePlayer {
                paused: Some(pause),
                ..Default::default()
            },
            true,
        )
        .await
    }

    /// Set the volume of the player.
    pub async fn set_volume(&self, mut volume: u16) -> LavalinkResult<player::Player> {
        volume = volume.min(1000);

        self.update_player(
            &http::UpdatePlayer {
                volume: Some(volume),
                ..Default::default()
            },
            true,
        )
        .await
    }

    /// Set the filters of the player.
    pub async fn set_filters(&self, filters: player::Filters) -> LavalinkResult<player::Player> {
        self.update_player(
            &http::UpdatePlayer {
                filters: Some(filters),
                ..Default::default()
            },
            true,
        )
        .await
    }

    /// Jump to a specific position in the currently playing track.
    pub async fn set_position(
        &self,
        position: std::time::Duration,
    ) -> LavalinkResult<player::Player> {
        self.update_player(
            &http::UpdatePlayer {
                position: Some(position.as_millis() as u64),
                ..Default::default()
            },
            true,
        )
        .await
    }

    /// Get the custom data provided when creating the player context.
    ///
    /// # Errors
    /// Returns `LavalinkError::InvalidDataType` if the type argument provided does not match the type of the data provided,
    /// or if no data was provided when creating the player context.
    pub fn data<Data: Send + Sync + 'static>(&self) -> LavalinkResult<std::sync::Arc<Data>> {
        self.user_data
            .clone()
            .downcast()
            .map_err(|_| LavalinkError::InvalidDataType)
    }
}

impl QueueRef {
    /// Get the current queue.
    ///
    /// Note: This clones the entire queue. Use the Stream implementation instead to iterate
    /// through it.
    pub async fn get_queue(&self) -> LavalinkResult<VecDeque<super::TrackInQueue>> {
        let (tx, rx) = oneshot::channel();

        self.send(super::QueueMessage::GetQueue(tx))?;

        Ok(rx.await?)
    }

    /// Get the track at the index.
    ///
    /// Note: This clones the track.
    pub async fn get_track(&self, index: usize) -> LavalinkResult<Option<super::TrackInQueue>> {
        let (tx, rx) = oneshot::channel();

        self.send(super::QueueMessage::GetTrack(index, tx))?;

        Ok(rx.await?)
    }

    /// Get the amount of tracks in the queue, AKA the queue length.
    pub async fn get_count(&self) -> LavalinkResult<usize> {
        let (tx, rx) = oneshot::channel();

        self.send(super::QueueMessage::GetCount(tx))?;

        Ok(rx.await?)
    }

    /// Add the track at the end of the queue.
    pub fn push_to_back(&self, track: impl Into<super::TrackInQueue>) -> LavalinkResult<()> {
        self.send(super::QueueMessage::PushToBack(track.into()))
    }

    /// Add the track at the start of the queue.
    pub fn push_to_front(&self, track: impl Into<super::TrackInQueue>) -> LavalinkResult<()> {
        self.send(super::QueueMessage::PushToFront(track.into()))
    }

    /// Insert the track at the given index.
    pub fn insert(
        &self,
        index: usize,
        track: impl Into<super::TrackInQueue>,
    ) -> LavalinkResult<()> {
        self.send(super::QueueMessage::Insert(index, track.into()))
    }

    /// Remove the track at the given index.
    pub fn remove(&self, index: usize) -> LavalinkResult<()> {
        self.send(super::QueueMessage::Remove(index))
    }

    /// Clear the queue.
    pub fn clear(&self) -> LavalinkResult<()> {
        self.send(super::QueueMessage::Clear)
    }

    /// Replace the entire queue with a new one.
    pub fn replace(&self, tracks: VecDeque<super::TrackInQueue>) -> LavalinkResult<()> {
        self.send(super::QueueMessage::Replace(tracks))
    }

    /// Append the list at the end of the current queue.
    pub fn append(&self, tracks: VecDeque<super::TrackInQueue>) -> LavalinkResult<()> {
        self.send(super::QueueMessage::Append(tracks))
    }

    /// Swap the track at the index with a new track.
    pub fn swap(&self, index: usize, track: impl Into<super::TrackInQueue>) -> LavalinkResult<()> {
        self.send(super::QueueMessage::Swap(index, track.into()))
    }

    /// Send messages to the queue to obtain tracks from it, or modify it.
    pub fn send(&self, queue_message: super::QueueMessage) -> LavalinkResult<()> {
        self.tx
            .send(super::PlayerMessage::QueueMessage(queue_message))?;
        Ok(())
    }
}

impl futures::Stream for QueueRef {
    type Item = super::TrackInQueue;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unsafe { Pin::new_unchecked(&mut *self.stream.lock().unwrap()) }.poll_next(cx)
    }
}
