use crate::model::*;

use std::collections::VecDeque;

use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub(crate) struct PlayerContextInner {
    pub guild_id: GuildId,
    pub queue: VecDeque<super::TrackInQueue>,
    pub player_data: player::Player,
    pub dummy: super::PlayerContext,
}

impl PlayerContextInner {
    pub async fn start(mut self, mut rx: UnboundedReceiver<super::PlayerMessage>) {
        tokio::spawn(async move {
            while let Some(x) = rx.recv().await {
                use super::PlayerMessage::*;

                match x {
                    UpdatePlayer(player) => self.player_data = player,
                    UpdatePlayerTrack(track) => self.player_data.track = track,
                    UpdatePlayerState(state) => self.player_data.state = state,
                    InsertToQueue(track) => self.queue.push_back(track),
                    ReplaceQueue(tracks) => self.queue = tracks,
                    AppendQueue(mut tracks) => self.queue.append(&mut tracks),
                    TrackFinished(should_continue) => {
                        if should_continue {
                            if let Err(why) = self.dummy.skip() {
                                error!(
                                    "Error sending skip message in player {}: {}",
                                    self.guild_id.0, why
                                );
                            }
                        }
                    }
                    StartTrack => {
                        if let Some(track) = self.queue.pop_front() {
                            if let Err(why) = self
                                .dummy
                                .update_player(&track.into_update_player(), false)
                                .await
                            {
                                error!(
                                    "Error sending update_player request in player {}: {}",
                                    self.guild_id.0, why
                                );
                            }
                        }
                    }
                    Close => rx.close(),
                };
            }
        });
    }
}
