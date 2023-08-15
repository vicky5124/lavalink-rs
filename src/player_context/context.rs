use std::collections::VecDeque;

use crate::client::LavalinkClient;
use crate::error::LavalinkResult;
use crate::model::*;

use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct PlayerContext {
    pub guild_id: GuildId,
    pub client: LavalinkClient,
    pub(crate) tx: UnboundedSender<super::PlayerMessage>,
    //pub user_data: Arc<RwLock<TypeMap>>
}

impl PlayerContext {
    pub fn close(self) -> LavalinkResult<()> {
        self.tx.send(super::PlayerMessage::Close)?;
        Ok(())
    }

    pub fn skip(&self) -> LavalinkResult<()> {
        self.tx.send(super::PlayerMessage::StartTrack)?;
        Ok(())
    }

    pub fn finish(&self, should_continue: bool) -> LavalinkResult<()> {
        self.tx
            .send(super::PlayerMessage::TrackFinished(should_continue))?;
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

    pub fn queue(&self, track: impl Into<super::TrackInQueue>) -> LavalinkResult<()> {
        self.tx
            .send(super::PlayerMessage::InsertToQueue(track.into()))?;
        Ok(())
    }

    pub fn replace_queue(
        &self,
        tracks: impl IntoIterator<Item = super::TrackInQueue>,
    ) -> LavalinkResult<()> {
        self.tx.send(super::PlayerMessage::ReplaceQueue(
            tracks.into_iter().collect(),
        ))?;
        Ok(())
    }

    pub fn append_queue(
        &self,
        tracks: impl IntoIterator<Item = super::TrackInQueue>,
    ) -> LavalinkResult<()> {
        self.tx.send(super::PlayerMessage::AppendQueue(
            tracks.into_iter().collect(),
        ))?;
        Ok(())
    }

    pub async fn get_queue(&self) -> LavalinkResult<VecDeque<crate::TrackInQueue>> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(super::PlayerMessage::GetQueue(tx))?;

        Ok(rx.await?)
    }

    pub async fn get_player(&self) -> LavalinkResult<player::Player> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(super::PlayerMessage::GetPlayer(tx))?;

        Ok(rx.await?)
    }

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

    pub async fn stop_now(&self) -> LavalinkResult<player::Player> {
        self.update_player(&http::UpdatePlayer::default(), false)
            .await
    }

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
