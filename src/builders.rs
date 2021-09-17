use crate::error::*;
use crate::gateway::LavalinkEventHandler;
use crate::model::*;
use crate::LavalinkClient;

use std::{net::SocketAddr, time::Duration};
//use serenity::model::guild::Region;

use tokio::time::sleep;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LavalinkClientBuilder {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub shard_count: u64,
    pub bot_id: UserId,
    pub is_ssl: bool,
    #[cfg(feature = "simple-gateway")]
    pub bot_token: String,
    #[cfg(feature = "simple-gateway")]
    pub start_gateway: bool,
    #[cfg(feature = "simple-gateway")]
    pub gateway_start_wait_time: Duration,
}

impl LavalinkClientBuilder {
    #[cfg(feature = "simple-gateway")]
    /// Builds the LavalinkClient.
    ///
    /// Default values:
    ///   - host: localhost
    ///   - port: 2333
    ///   - password: youshallnotpass
    ///   - shard_count: 1
    ///   - is_ssl: false
    ///   - bot_id: <required parameter>
    ///   - bot_token: <required parameter>
    ///   - start_gateway: true
    ///   - gateway_start_wait_time: 6 seconds
    pub fn new(bot_id: impl Into<UserId>, bot_token: impl Into<String>) -> Self {
        Self {
            host: "localhost".to_string(),
            port: 2333,
            password: "youshallnotpass".to_string(),
            shard_count: 1,
            bot_id: bot_id.into(),
            bot_token: bot_token.into(),
            start_gateway: true,
            gateway_start_wait_time: Duration::from_secs(6),
            ..Default::default()
        }
    }

    #[cfg(not(feature = "simple-gateway"))]
    ///
    /// Builds the LavalinkClient.
    ///
    /// Default values:
    ///   - host: localhost
    ///   - port: 2333
    ///   - password: youshallnotpass
    ///   - shard_count: 1
    ///   - is_ssl: false
    ///   - bot_id: <required parameter>
    pub fn new(bot_id: impl Into<UserId>) -> Self {
        Self {
            host: "localhost".to_string(),
            port: 2333,
            password: "youshallnotpass".to_string(),
            shard_count: 1,
            bot_id: bot_id.into(),
            ..Default::default()
        }
    }

    /// Sets the host.
    pub fn set_host(&mut self, host: impl ToString) -> &mut Self {
        self.host = host.to_string();
        self
    }

    /// Sets the port.
    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    /// Sets the host and port from an address.
    pub fn set_addr(&mut self, addr: impl Into<SocketAddr>) -> &mut Self {
        let addr = addr.into();

        self.host = addr.ip().to_string();
        self.port = addr.port();

        self
    }

    /// Sets the number of shards.
    pub fn set_shard_count(&mut self, shard_count: u64) -> &mut Self {
        self.shard_count = shard_count;
        self
    }

    /// Sets the ID of the bot.
    pub fn set_bot_id<U: Into<UserId>>(&mut self, bot_id: U) -> &mut Self {
        self.bot_id = bot_id.into();
        self
    }

    /// Sets the token of the bot.
    #[cfg(feature = "simple-gateway")]
    pub fn set_bot_token<U: Into<String>>(&mut self, bot_token: U) -> &mut Self {
        self.bot_token = bot_token.into();
        self
    }

    /// Sets if the lavalink server is behind SSL
    pub fn set_is_ssl(&mut self, is_ssl: bool) -> &mut Self {
        self.is_ssl = is_ssl;
        self
    }

    /// Sets the lavalink password.
    pub fn set_password(&mut self, password: impl ToString) -> &mut Self {
        self.password = password.to_string();
        self
    }

    /// Sets if the discord gateway for voice connections should start or not.
    #[cfg(feature = "simple-gateway")]
    pub fn set_start_gateway(&mut self, start_gateway: bool) -> &mut Self {
        self.start_gateway = start_gateway;
        self
    }

    /// Sets the time to wait before starting the first discord gateway connection.
    #[cfg(feature = "simple-gateway")]
    pub fn set_gateway_start_wait_time(&mut self, time: Duration) -> &mut Self {
        self.gateway_start_wait_time = time;
        self
    }

    /// Build the builder into a Client
    pub async fn build(
        &self,
        handler: impl LavalinkEventHandler + Send + Sync + 'static,
    ) -> Result<LavalinkClient, LavalinkError> {
        LavalinkClient::new(self, handler).await
    }
}

#[derive(Clone)]
pub struct PlayParameters {
    pub track: Track,
    pub replace: bool,
    pub start: u64,
    pub finish: u64,
    pub guild_id: u64,
    pub requester: Option<UserId>,
    pub client: LavalinkClient,
}

impl PlayParameters {
    /// Starts playing the track.
    pub async fn start(&self) -> LavalinkResult<()> {
        let payload = crate::model::Play {
            track: self.track.track.clone(),
            no_replace: !self.replace,
            start_time: self.start,
            end_time: if self.finish == 0 {
                None
            } else {
                Some(self.finish)
            },
        };

        let mut client = self.client.inner.lock().await;

        SendOpcode::Play(payload)
            .send(self.guild_id, &mut client.socket_write)
            .await?;

        Ok(())
    }

    /// Adds the track to the node queue.
    ///
    /// If there's no queue loop running, this will start one up, and add it to the running loops
    /// on [`LavalinkClient.loops`].
    ///
    /// Needs for [`LavalinkClient::create_session`] to be called first.
    ///
    /// [`LavalinkClient.loops`]: crate::LavalinkClientInner::loops
    /// [`LavalinkClient::create_session`]: crate::LavalinkClient::create_session
    pub async fn queue(&self) -> LavalinkResult<()> {
        let track = crate::model::TrackQueue {
            track: self.track.clone(),
            start_time: self.start,
            end_time: if self.finish == 0 {
                None
            } else {
                Some(self.finish)
            },
            requester: self.requester,
        };

        let client = self.client.clone();

        let client_lock = client.inner.lock().await;

        if !client_lock.loops.contains(&self.guild_id) {
            let guild_id = self.guild_id;

            if let Some(mut node) = client_lock.nodes.get_mut(&guild_id) {
                if !node.is_on_loops {
                    node.is_on_loops = true;
                } else {
                    let mut node = client_lock.nodes.get_mut(&self.guild_id).unwrap();
                    node.queue.push(track);

                    return Ok(());
                }
            } else {
                return Err(LavalinkError::NoSessionPresent);
            }

            client_lock.loops.insert(guild_id);

            {
                let mut node = client_lock.nodes.get_mut(&guild_id).unwrap();
                node.queue.push(track.clone());
            }

            drop(client_lock);

            let client_clone = client.clone();

            tokio::spawn(async move {
                loop {
                    let mut client_lock = client_clone.inner.lock().await;

                    if let Some(mut node) = client_lock.nodes.clone().get_mut(&guild_id) {
                        if !node.queue.is_empty() && node.now_playing.is_none() {
                            let track = node.queue[0].clone();

                            node.now_playing = Some(node.queue[0].clone());

                            let payload = crate::model::Play {
                                track: track.track.track.clone(), // track
                                no_replace: false,
                                start_time: track.start_time,
                                end_time: track.end_time,
                            };

                            if let Err(why) = crate::model::SendOpcode::Play(payload)
                                .send(guild_id, &mut client_lock.socket_write)
                                .await
                            {
                                eprintln!("Error playing queue on guild {} -> {}", guild_id, why);
                            }
                        }
                    } else {
                        //client.loops.remove(guild_id);
                        break;
                    }

                    drop(client_lock);

                    sleep(Duration::from_secs(1)).await;
                }
            });

            return Ok(());
        }

        let mut node = client_lock.nodes.get_mut(&self.guild_id).unwrap();
        node.queue.push(track);

        Ok(())
    }

    /// Sets the person that requested the song
    pub fn requester(&mut self, requester: impl Into<UserId>) -> &mut Self {
        self.requester = Some(requester.into());
        self
    }

    /// Sets if the current playing track should be replaced with this new one.
    pub fn replace(&mut self, replace: bool) -> &mut Self {
        self.replace = replace;
        self
    }

    /// Sets the time the track will start at.
    pub fn start_time(&mut self, start: Duration) -> &mut Self {
        self.start = start.as_millis() as u64;
        self
    }

    /// Sets the time the track will finish at.
    pub fn finish_time(&mut self, finish: Duration) -> &mut Self {
        self.finish = finish.as_millis() as u64;
        self
    }
}
