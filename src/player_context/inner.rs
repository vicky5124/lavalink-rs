use crate::model::*;

use std::collections::VecDeque;

use tokio::sync::mpsc::UnboundedReceiver;

#[cfg_attr(not(feature = "user-data"), derive(Debug))]
pub(crate) struct PlayerContextInner {
    pub guild_id: GuildId,
    pub queue: VecDeque<super::TrackInQueue>,
    pub player_data: player::Player,
    pub dummy: super::PlayerContext,
    pub last_should_continue: bool,
}

impl PlayerContextInner {
    pub async fn start(mut self, mut rx: UnboundedReceiver<super::PlayerMessage>) {
        tokio::spawn(async move {
            while let Some(x) = rx.recv().await {
                use super::PlayerMessage::*;

                match x {
                    GetPlayer(tx) => {
                        if let Err(why) = tx.send(self.player_data.clone()) {
                            error!(
                                "Error sending player back to the player {}: {}",
                                self.guild_id.0, why
                            );
                        }
                    }
                    UpdatePlayer(player) => self.player_data = player,
                    UpdatePlayerTrack(track) => self.player_data.track = track,
                    UpdatePlayerState(state) => self.player_data.state = state,

                    GetQueue(tx) => {
                        if let Err(why) = tx.send(self.queue.clone()) {
                            error!(
                                "Error sending queue back to the player {}: {}",
                                self.guild_id.0, why
                            );
                        }
                    }
                    SetQueue(queue_message) => {
                        self.queue_init().await;

                        use super::QueueMessage::*;

                        match queue_message {
                            PushToBack(track) => {
                                self.queue.push_back(track);
                            }
                            PushToFront(track) => {
                                self.queue.push_front(track);
                            }
                            Insert(index, track) => {
                                self.queue.insert(index, track);
                            }
                            Remove(index) => {
                                self.queue.remove(index);
                            }
                            Clear => {
                                self.queue.clear();
                            }
                            Replace(tracks) => {
                                self.queue = tracks;
                            }
                            Append(mut tracks) => {
                                self.queue.append(&mut tracks);
                            }
                        }
                    }

                    TrackFinished(should_continue) => {
                        self.last_should_continue = should_continue;

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
                        } else {
                            if let Err(why) = self.dummy.stop_now().await {
                                error!(
                                    "Error sending stop request in player {}: {}",
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

    async fn queue_init(&self) {
        if self.last_should_continue && self.player_data.track.is_none() {
            if let Err(why) = self.dummy.skip() {
                error!(
                    "Error sending skip message in player {}: {}",
                    self.guild_id.0, why
                );
            }
        }
    }
}
