use crate::error::{LavalinkError, LavalinkResult};
use crate::model::{ChannelId, ConnectionInfo, GuildId};
use crate::LavalinkClient;

use tokio::time::{sleep, Duration};

pub async fn join(
    lavalink: &LavalinkClient,
    guild_id: impl Into<GuildId>,
    channel_id: impl Into<ChannelId>,
) -> LavalinkResult<ConnectionInfo> {
    let guild_id = guild_id.into();
    let channel_id = channel_id.into();

    call(
        lavalink,
        format!(
            r#"{{
                "op": 4,
                "d": {{
                    "guild_id": "{}",
                    "channel_id": "{}",
                    "self_mute": false,
                    "self_deaf": true
                }}
            }}"#,
            guild_id.0, channel_id.0
        ),
    )
    .await;

    let connections = lavalink.discord_gateway_connections().await;

    let mut check_count = 0;

    while check_count < 10 {
        if let Some(d) = connections.get(&guild_id) {
            if d.token.is_some() && d.endpoint.is_some() && d.session_id.is_some() {
                return Ok(d.clone());
            }
        }

        sleep(Duration::from_millis(500)).await;

        check_count += 1;
    }

    return Err(LavalinkError::Timeout);
}

pub async fn leave(lavalink: &LavalinkClient, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
    let guild_id = guild_id.into();

    call(
        lavalink,
        format!(
            r#"{{
                "op": 4,
                "d": {{
                    "guild_id": "{}",
                    "channel_id": null,
                    "self_mute": false,
                    "self_deaf": true
                }}
            }}"#,
            guild_id.0,
        ),
    )
    .await;

    let connections = lavalink.discord_gateway_connections().await;

    let mut check_count = 0;

    while check_count < 10 {
        if connections.get(&guild_id).is_none() {
            return Ok(());
        }

        sleep(Duration::from_millis(500)).await;

        check_count += 1;
    }

    return Err(LavalinkError::Timeout);
}

pub async fn call(lavalink: &LavalinkClient, message: String) {
    lavalink
        .discord_gateway_data()
        .await
        .lock()
        .await
        .sender
        .send(message)
        .unwrap();
}

// pub async fn raw_handle_event_voice_server_update(
//     lavalink: LavalinkClient,
//     guild_id: Option<GuildId>,
//     channel_id: Option<ChannelId>,
//     endpoint: Option<String>,
//     token: String,
// ) {
//     // ...
// }
//
// pub async fn raw_handle_event_voice_state_update(
//     lavalink: LavalinkClient,
//     guild_id: Option<impl Into<GuildId>>,
//     channel_id: Option<impl Into<ChannelId>>,
//     user_id: impl Into<UserId>,
//     session_id: String,
//     //token: Option<String>,
// ) {
//     let gateway_lock = {
//         let client = lavalink.inner.lock().await;
//         client.discord_gateway.clone()
//     };
//
//     if user_id.into() != gateway_lock.lock().await.bot_id {
//         return;
//     }
// }
