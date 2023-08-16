#[macro_use]
extern crate tracing;

use lavalink_rs::{model::events, LavalinkClient, NodeBuilder, SearchEngines, TrackLoadData};

use hook::hook;
use itertools::Itertools;
use poise::serenity_prelude as serenity;
use serenity::Mentionable;
use songbird::SerenityInit;

struct Data {
    lavalink: LavalinkClient,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Play a song in the voice channel you are connected in.
#[poise::command(slash_command, prefix_command)]
async fn play(
    ctx: Context<'_>,
    #[description = "Search term or URL"]
    #[rest]
    term: String,
) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let lava_client = ctx.data().lavalink.clone();

    if manager.get(guild_id).is_none() || lava_client.get_player_context(guild_id).is_none() {
        let channel_id = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);

        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                ctx.say("Not in a voice channel").await?;

                return Ok(());
            }
        };

        let (_, handler) = manager.join_gateway(guild_id, connect_to).await;

        match handler {
            Ok(connection_info) => {
                lava_client.create_player(guild_id, connection_info).await?;

                ctx.say(format!("Joined {}", connect_to.mention())).await?;
            }
            Err(why) => {
                ctx.say(format!("Error joining the channel: {}", why))
                    .await?;
                return Ok(());
            }
        }
    }

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

    let tracks: Vec<lavalink_rs::TrackInQueue> = match loaded_tracks.data {
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

    player.append_queue(tracks)?;

    Ok(())
}

/// Add a song to the queue
#[poise::command(slash_command, prefix_command)]
async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let lava_client = ctx.data().lavalink.clone();

    let Some(player) = lava_client.get_player_context(guild_id) else {
        ctx.say("Join the bot to a voice channel first.").await?;
        return Ok(());
    };

    let queue = player.get_queue().await?;
    let player_data = player.get_player().await?;

    let max = queue.len().min(9);
    let queue_message = queue
        .range(0..max)
        .enumerate()
        .map(|(idx, x)| {
            if let Some(uri) = &x.track.info.uri {
                format!(
                    "{} -> [{} - {}](<{}>)",
                    idx + 1,
                    x.track.info.author,
                    x.track.info.title,
                    uri
                )
            } else {
                format!(
                    "{} -> {} - {}",
                    idx + 1,
                    x.track.info.author,
                    x.track.info.title
                )
            }
        })
        .join("\n");

    let now_playing_message = if let Some(track) = player_data.track {
        if let Some(uri) = &track.info.uri {
            format!(
                "Now playing: [{} - {}](<{}>)",
                track.info.author, track.info.title, uri
            )
        } else {
            format!("Now playing: {} - {}", track.info.author, track.info.title)
        }
    } else {
        "Now playing: nothing".to_string()
    };

    ctx.say(format!("{}\n\n{}", now_playing_message, queue_message))
        .await?;

    Ok(())
}

/// Leave the current voice channel.
#[poise::command(slash_command, prefix_command)]
async fn leave(ctx: Context<'_>) -> Result<(), Error> {
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

/// Test
#[poise::command(prefix_command)]
async fn test(ctx: Context<'_>) -> Result<(), Error> {
    //use std::time::Duration;

    ctx.say("AAAAAAAAAAA").await?;

    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let lava_client = ctx.data().lavalink.clone();

    let player = lava_client.get_player_context(guild_id).unwrap();
    player.skip()?;

    //dbg!(lava_client.info(guild_id).await?);
    //dbg!(lava_client.stats(guild_id).await?);
    //dbg!(lava_client.version(guild_id).await?);
    //dbg!(lava_client.decode_track(guild_id, "QAAAxAMAEU5vc2VibGVlZCBTZWN0aW9uABtTaWdodGxlc3MgaW4gU2hhZG93IC0gVG9waWMAAAAAAAJxAAALTFdNUi1rY3dHLTgAAQAraHR0cHM6Ly93d3cueW91dHViZS5jb20vd2F0Y2g/dj1MV01SLWtjd0ctOAEAOmh0dHBzOi8vaS55dGltZy5jb20vdmlfd2VicC9MV01SLWtjd0ctOC9tYXhyZXNkZWZhdWx0LndlYnAAAAd5b3V0dWJlAAAAAAACb1w=").await?);
    //dbg!(lava_client.decode_tracks(guild_id, &["QAAAxAMAEU5vc2VibGVlZCBTZWN0aW9uABtTaWdodGxlc3MgaW4gU2hhZG93IC0gVG9waWMAAAAAAAJxAAALTFdNUi1rY3dHLTgAAQAraHR0cHM6Ly93d3cueW91dHViZS5jb20vd2F0Y2g/dj1MV01SLWtjd0ctOAEAOmh0dHBzOi8vaS55dGltZy5jb20vdmlfd2VicC9MV01SLWtjd0ctOC9tYXhyZXNkZWZhdWx0LndlYnAAAAd5b3V0dWJlAAAAAAACb1w=".to_string()]).await?);
    //dbg!(lava_client.get_player(guild_id).await?);
    //dbg!(lava_client.get_players(guild_id).await?);

    //lava_client.set_position(guild_id, Duration::from_secs(120)).await?;
    //tokio::time::sleep(Duration::from_secs(2)).await;
    //lava_client.set_pause(guild_id, true).await?;
    //tokio::time::sleep(Duration::from_secs(2)).await;
    //lava_client.set_pause(guild_id, false).await?;
    //tokio::time::sleep(Duration::from_secs(2)).await;
    //lava_client.set_volume(guild_id, 50).await?;
    //tokio::time::sleep(Duration::from_secs(2)).await;
    //lava_client.set_volume(guild_id, 100).await?;

    ctx.say("all good!").await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info,lavalink_rs=trace");
    tracing_subscriber::fmt::init();

    let framework = poise::Framework::builder()
        .client_settings(|c| c.register_songbird())
        .options(poise::FrameworkOptions {
            commands: vec![play(), queue(), leave(), test()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(",".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::all())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let events = events::Events {
                    raw: Some(raw_event),
                    ready: Some(ready_event),
                    ..Default::default()
                };

                let node_local = NodeBuilder {
                    hostname: "localhost:2333".to_string(),
                    is_ssl: false,
                    events: events::Events::default(),
                    password: env!("LAVALINK_PASSWORD").to_string(),
                    user_id: ctx.cache.current_user_id().into(),
                    session_id: None,
                };

                let client = LavalinkClient::new(events, vec![node_local]);

                client.start().await;

                Ok(Data { lavalink: client })
            })
        });

    framework.run().await.unwrap();
}

#[hook]
async fn raw_event(_: LavalinkClient, session_id: String, event: &serde_json::Value) {
    if event["op"].as_str() == Some("event") || event["op"].as_str() == Some("playerUpdate") {
        info!("{:?} -> {:?}", session_id, event);
    }
}

#[hook]
async fn ready_event(client: LavalinkClient, session_id: String, event: &events::Ready) {
    client.delete_all_players().await.unwrap();
    info!("{:?} -> {:?}", session_id, event);
}
