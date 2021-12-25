use crate::error::{LavalinkError, LavalinkResult};
use crate::gateway::call_discord_gateway;
use crate::model::{ChannelId, ConnectionInfo, GuildId, UserId};
use crate::LavalinkClient;

use tokio::time::{sleep, Duration};

pub async fn join(
    lavalink: &LavalinkClient,
    guild_id: impl Into<GuildId>,
    channel_id: impl Into<ChannelId>,
) -> LavalinkResult<ConnectionInfo> {
    let guild_id = guild_id.into();
    let channel_id = channel_id.into();

    call_discord_gateway(
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
    );

    wait_for_full_connection_info_insert(lavalink, guild_id, None).await
}

pub async fn leave(lavalink: &LavalinkClient, guild_id: impl Into<GuildId>) -> LavalinkResult<()> {
    let guild_id = guild_id.into();

    call_discord_gateway(
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
    );

    wait_for_connection_info_remove(lavalink, guild_id, None).await
}

pub async fn wait_for_full_connection_info_insert(
    lavalink: &LavalinkClient,
    guild_id: impl Into<GuildId>,
    event_count: Option<usize>,
) -> LavalinkResult<ConnectionInfo> {
    let guild_id = guild_id.into();
    let connections = lavalink.discord_gateway_connections();

    let mut check_count = 0;

    while check_count <= event_count.unwrap_or(10) {
        if let Some(d) = connections.get(&guild_id) {
            if d.token.is_some() && d.endpoint.is_some() && d.session_id.is_some() {
                return Ok(d.clone());
            }
        }

        sleep(Duration::from_millis(500)).await;

        check_count += 1;
    }

    Err(LavalinkError::Timeout)
}

pub async fn wait_for_connection_info_remove(
    lavalink: &LavalinkClient,
    guild_id: impl Into<GuildId>,
    event_count: Option<usize>,
) -> LavalinkResult<()> {
    let guild_id = guild_id.into();
    let connections = lavalink.discord_gateway_connections();

    let mut check_count = 0;

    while check_count <= event_count.unwrap_or(10) {
        if connections.get(&guild_id).is_none() {
            return Ok(());
        }

        sleep(Duration::from_millis(500)).await;

        check_count += 1;
    }

    Err(LavalinkError::Timeout)
}

pub async fn raw_handle_event_voice_server_update(
    lavalink: &LavalinkClient,
    guild_id: impl Into<GuildId>,
    endpoint: String,
    token: String,
) {
    let guild_id = guild_id.into();

    let endpoint = if endpoint.starts_with("wss://") {
        endpoint.strip_prefix("wss://").unwrap().to_string()
    } else {
        endpoint
    };

    let connections = lavalink.discord_gateway_data().lock().connections.clone();

    if let Some(mut connection) = connections.get_mut(&guild_id) {
        connection.guild_id = Some(guild_id);
        connection.endpoint = Some(endpoint);
        connection.token = Some(token);
    } else {
        connections.insert(
            guild_id,
            ConnectionInfo {
                guild_id: Some(guild_id),
                endpoint: Some(endpoint),
                token: Some(token),
                ..Default::default()
            },
        );
    };

    let connection = connections.get(&guild_id).unwrap().clone();
    let lavalink = lavalink.clone();

    tokio::spawn(async move {
        if connection.endpoint.is_some() && connection.session_id.is_some() {
            debug!("(Voice Server Update) Call pause");
            if let Err(why) = lavalink.pause(guild_id).await {
                error!("Error when pausing on voice_server_update: {}", why);
            }

            sleep(Duration::from_millis(100)).await;

            debug!("(Voice Server Update) Call create_session");
            if let Err(why) = lavalink.create_session(&connection).await {
                error!(
                    "Error when creating a session on voice_server_update: {}",
                    why
                );
            }

            sleep(Duration::from_millis(1000)).await;

            debug!("(Voice Server Update) Call resume");
            if let Err(why) = lavalink.resume(guild_id).await {
                error!("Error when resuming on voice_server_update: {}", why);
            }
        }
    });
}

pub fn raw_handle_event_voice_state_update(
    lavalink: &LavalinkClient,
    guild_id: impl Into<GuildId>,
    channel_id: Option<impl Into<ChannelId>>,
    user_id: impl Into<UserId>,
    session_id: String,
) {
    let guild_id = guild_id.into();
    let user_id = user_id.into();
    let channel_id = channel_id.map(|c| c.into());

    let gateway_data = lavalink.discord_gateway_data();
    let ws_data = gateway_data.lock();

    if user_id != ws_data.bot_id {
        return;
    }

    let connections = ws_data.connections.clone();

    drop(ws_data);

    if channel_id.is_none() {
        connections.remove(&guild_id);
        return;
    }

    if let Some(mut connection) = connections.get_mut(&guild_id) {
        connection.session_id = Some(session_id);
        connection.channel_id = channel_id;
    } else {
        connections.insert(
            guild_id,
            ConnectionInfo {
                guild_id: Some(guild_id),
                session_id: Some(session_id),
                channel_id,
                ..Default::default()
            },
        );
    };
}
