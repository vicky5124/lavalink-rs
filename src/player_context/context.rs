use crate::client::LavalinkClient;
use crate::error::LavalinkResult;
use crate::model::*;

use std::collections::VecDeque;

use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
#[cfg_attr(not(feature = "user-data"), derive(Debug))]
/// The player context.
pub struct PlayerContext {
    pub guild_id: GuildId,
    pub client: LavalinkClient,
    pub(crate) tx: UnboundedSender<super::PlayerMessage>,
    #[cfg(feature = "user-data")]
    pub user_data: std::sync::Arc<parking_lot::RwLock<typemap_rev::TypeMap>>,
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

    /// Add a track to the queue.
    pub fn queue(&self, track: impl Into<super::TrackInQueue>) -> LavalinkResult<()> {
        self.tx
            .send(super::PlayerMessage::InsertToQueue(track.into()))?;
        Ok(())
    }

    /// Replace the entire queue with another one.
    pub fn replace_queue(
        &self,
        tracks: impl IntoIterator<Item = super::TrackInQueue>,
    ) -> LavalinkResult<()> {
        self.tx.send(super::PlayerMessage::ReplaceQueue(
            tracks.into_iter().collect(),
        ))?;
        Ok(())
    }

    /// Append another queue to the end of the current one.
    pub fn append_queue(
        &self,
        tracks: impl IntoIterator<Item = super::TrackInQueue>,
    ) -> LavalinkResult<()> {
        self.tx.send(super::PlayerMessage::AppendQueue(
            tracks.into_iter().collect(),
        ))?;
        Ok(())
    }

    /// Get the current queue.
    pub async fn get_queue(&self) -> LavalinkResult<VecDeque<super::TrackInQueue>> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(super::PlayerMessage::GetQueue(tx))?;

        Ok(rx.await?)
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
        let node = self.client.get_node_for_guild(self.guild_id);

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
        self.update_player(&http::UpdatePlayer::default(), false)
            .await
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
                position: Some(position.as_millis()),
                ..Default::default()
            },
            true,
        )
        .await
    }
}
