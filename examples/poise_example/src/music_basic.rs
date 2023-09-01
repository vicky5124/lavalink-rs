use crate::Context;
use crate::Error;

use lavalink_rs::prelude::*;

use poise::serenity_prelude as serenity;
use serenity::Mentionable;

async fn _join(
    ctx: &Context<'_>,
    guild_id: serenity::GuildId,
    channel_id: Option<serenity::ChannelId>,
) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let lava_client = ctx.data().lavalink.clone();

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if lava_client.get_player_context(guild_id).is_none() {
        let connect_to = match channel_id {
            Some(x) => x,
            None => {
                let user_channel_id = guild
                    .voice_states
                    .get(&ctx.author().id)
                    .and_then(|voice_state| voice_state.channel_id);

                match user_channel_id {
                    Some(channel) => channel,
                    None => {
                        ctx.say("Not in a voice channel").await?;

                        return Err("Not in a voice channel".into());
                    }
                }
            }
        };

        let (_, handler) = manager.join_gateway(guild_id, connect_to).await;

        match handler {
            Ok(connection_info) => {
                lava_client
                    .create_player_context(guild_id, connection_info)
                    .await?;

                ctx.say(format!("Joined {}", connect_to.mention())).await?;

                return Ok(());
            }
            Err(why) => {
                ctx.say(format!("Error joining the channel: {}", why))
                    .await?;
                return Err(why.into());
            }
        }
    }

    Ok(())
}

/// Play a song in the voice channel you are connected in.
#[poise::command(slash_command, prefix_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Search term or URL"]
    #[rest]
    term: String,
) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    _join(&ctx, guild_id, None).await?;

    let lava_client = ctx.data().lavalink.clone();

    let Some(player) = lava_client.get_player_context(guild_id) else {
        ctx.say("Join the bot to a voice channel first.").await?;
        return Ok(());
    };

    let query = if term.starts_with("http") {
        term
    } else {
        SearchEngines::YouTube.to_query(&term)?
    };

    let loaded_tracks = lava_client.load_tracks(guild_id, &query).await?;

    let mut playlist_info = None;

    let tracks: Vec<TrackInQueue> = match loaded_tracks.data {
        Some(TrackLoadData::Track(x)) => vec![x.into()],
        Some(TrackLoadData::Search(x)) => vec![x[0].clone().into()],
        Some(TrackLoadData::Playlist(x)) => {
            playlist_info = Some(x.info);
            x.tracks.iter().map(|x| x.into()).collect()
        }

        _ => {
            ctx.say(format!("{:?}", loaded_tracks)).await?;
            return Ok(());
        }
    };

    if let Some(info) = playlist_info {
        ctx.say(format!("Added playlist to queue: {}", info.name,))
            .await?;
    } else {
        let track = &tracks[0].track;

        if let Some(uri) = &track.info.uri {
            ctx.say(format!(
                "Added to queue: [{} - {}](<{}>)",
                track.info.author, track.info.title, uri
            ))
            .await?;
        } else {
            ctx.say(format!(
                "Added to queue: {} - {}",
                track.info.author, track.info.title
            ))
            .await?;
        }
    }

    player.set_queue(QueueMessage::Append(tracks.into()))?;

    if let Ok(player_context) = player.get_player().await {
        if player_context.track.is_none() && player.get_queue().await.is_ok_and(|x| !x.is_empty()) {
            player.skip()?;
        }
    }

    Ok(())
}

/// Join the specified voice channel or the one you are currently in.
#[poise::command(slash_command, prefix_command)]
pub async fn join(
    ctx: Context<'_>,
    #[description = "The channel ID to join to."]
    #[channel_types("Voice")]
    channel_id: Option<serenity::ChannelId>,
) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    _join(&ctx, guild_id, channel_id).await?;

    Ok(())
}
/// Leave the current voice channel.
#[poise::command(slash_command, prefix_command)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let lava_client = ctx.data().lavalink.clone();

    lava_client.delete_player(guild_id).await?;

    if manager.get(guild_id).is_some() {
        manager.remove(guild_id).await?;
    }

    ctx.say("Left voice channel.").await?;

    Ok(())
}
