use super::*;

pub(crate) enum ClientMessage {
    GetConnectionInfo(
        GuildId,
        std::time::Duration,
        oneshot::Sender<Result<player::ConnectionInfo, tokio::time::error::Elapsed>>,
    ),
    ServerUpdate(GuildId, String, Option<String>), // guild_id, token, endpoint
    StateUpdate(GuildId, Option<ChannelId>, UserId, String), // guild_id, channel_id, user_id, session_id
}
