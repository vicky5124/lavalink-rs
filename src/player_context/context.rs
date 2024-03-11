use crate::client::LavalinkClient;
use crate::error::{LavalinkError, LavalinkResult};
use crate::model::*;

use std::collections::VecDeque;

use reqwest::Method;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyo3::pyclass(sequence))]
/// The player context.
pub struct PlayerContext {
    pub guild_id: GuildId,
    pub client: LavalinkClient,
    pub(crate) tx: UnboundedSender<super::PlayerMessage>,
    pub(crate) user_data: std::sync::Arc<dyn std::any::Any + Send + Sync>,
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
        self.set_queue(super::QueueMessage::PushToBack(track.into()))
    }

    /// Get the current queue.
    pub async fn get_queue(&self) -> LavalinkResult<VecDeque<super::TrackInQueue>> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(super::PlayerMessage::GetQueue(tx))?;

        Ok(rx.await?)
    }

    /// Modify the queue in specific ways.
    pub fn set_queue(&self, queue_message: super::QueueMessage) -> LavalinkResult<()> {
        self.tx
            .send(super::PlayerMessage::SetQueue(queue_message))?;
        Ok(())
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
                encoded_track: Some(track.encoded.to_string()),
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
                encoded_track: Some(track.encoded.to_string()),
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

        let path = format!(
            "/v4/sessions/{}/players/{}",
            &node.session_id.load(),
            self.guild_id.0
        );

        let result = node
            .http
            .raw_request(
                Method::PATCH,
                path,
                &serde_json::json!({"encodedTrack": null}),
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
