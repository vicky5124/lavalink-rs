#[macro_use]
extern crate tracing;

use std::time::Duration;

use lavalink_rs::client::LavalinkClient;
use lavalink_rs::model::player::ConnectionInfo;
use lavalink_rs::model::track::TrackLoadData;
use lavalink_rs::model::*;
use lavalink_rs::node;

use hook::hook;
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
    let lavalink_guild_id = GuildId(guild_id.0);

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let lava_client = ctx.data().lavalink.clone();

    if manager.get(guild_id).is_none() {
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
                lava_client
                    .create_player(
                        lavalink_guild_id,
                        &ConnectionInfo {
                            endpoint: connection_info.endpoint,
                            token: connection_info.token,
                            session_id: connection_info.session_id,
                        },
                    )
                    .await?;

                ctx.say(format!("Joined {}", connect_to.mention())).await?;
            }
            Err(why) => {
                ctx.say(format!("Error joining the channel: {}", why))
                    .await?;
                return Ok(());
            }
        }
    }

    let loaded_tracks = lava_client.load_tracks(lavalink_guild_id, &term).await?;

    let track = match &loaded_tracks.data {
        Some(TrackLoadData::Track(x)) => x,
        Some(TrackLoadData::Playlist(x)) => &x.tracks[0],
        Some(TrackLoadData::Search(x)) => &x[0],

        _ => {
            ctx.say(format!("{:?}", loaded_tracks)).await?;
            return Ok(());
        }
    };

    lava_client.play_now(lavalink_guild_id, track).await?;

    if let Some(uri) = &track.info.uri {
        ctx.say(format!(
            "Playing: [{} - {}](<{}>)",
            track.info.author, track.info.title, uri
        ))
        .await?;
    } else {
        ctx.say(format!(
            "Playing: {} - {}",
            track.info.author, track.info.title
        ))
        .await?;
    }

    Ok(())
}

/// Leave the current voice channel.
#[poise::command(slash_command, prefix_command)]
async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;
    let lavalink_guild_id = GuildId(guild_id.0);

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let lava_client = ctx.data().lavalink.clone();

    lava_client.delete_player(lavalink_guild_id).await?;

    if manager.get(guild_id).is_some() {
        manager.remove(guild_id).await?;
    }

    ctx.say("Left voice channel.").await?;

    Ok(())
}

/// Test
#[poise::command(prefix_command)]
async fn test(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;
    let lavalink_guild_id = GuildId(guild_id.0);

    let lava_client = ctx.data().lavalink.clone();

    //dbg!(lava_client.info(lavalink_guild_id).await?);
    //dbg!(lava_client.stats(lavalink_guild_id).await?);
    //dbg!(lava_client.version(lavalink_guild_id).await?);
    //dbg!(lava_client.decode_track(lavalink_guild_id, "QAAAxAMAEU5vc2VibGVlZCBTZWN0aW9uABtTaWdodGxlc3MgaW4gU2hhZG93IC0gVG9waWMAAAAAAAJxAAALTFdNUi1rY3dHLTgAAQAraHR0cHM6Ly93d3cueW91dHViZS5jb20vd2F0Y2g/dj1MV01SLWtjd0ctOAEAOmh0dHBzOi8vaS55dGltZy5jb20vdmlfd2VicC9MV01SLWtjd0ctOC9tYXhyZXNkZWZhdWx0LndlYnAAAAd5b3V0dWJlAAAAAAACb1w=").await?);
    //dbg!(lava_client.decode_tracks(lavalink_guild_id, &["QAAAxAMAEU5vc2VibGVlZCBTZWN0aW9uABtTaWdodGxlc3MgaW4gU2hhZG93IC0gVG9waWMAAAAAAAJxAAALTFdNUi1rY3dHLTgAAQAraHR0cHM6Ly93d3cueW91dHViZS5jb20vd2F0Y2g/dj1MV01SLWtjd0ctOAEAOmh0dHBzOi8vaS55dGltZy5jb20vdmlfd2VicC9MV01SLWtjd0ctOC9tYXhyZXNkZWZhdWx0LndlYnAAAAd5b3V0dWJlAAAAAAACb1w=".to_string()]).await?);
    //dbg!(lava_client.get_player(lavalink_guild_id).await?);
    //dbg!(lava_client.get_players(lavalink_guild_id).await?);

    lava_client
        .set_position(lavalink_guild_id, Duration::from_secs(120))
        .await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    lava_client.set_pause(lavalink_guild_id, true).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    lava_client.set_pause(lavalink_guild_id, false).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    lava_client.set_volume(lavalink_guild_id, 50).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    lava_client.set_volume(lavalink_guild_id, 100).await?;

    ctx.say("all good!").await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info,lavalink-rs=trace");
    tracing_subscriber::fmt::init();

    let framework = poise::Framework::builder()
        .client_settings(|c| c.register_songbird())
        .options(poise::FrameworkOptions {
            commands: vec![play(), leave(), test()],
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

                let node_local = node::NodeBuilder {
                    hostname: "localhost:2333".to_string(),
                    is_ssl: false,
                    events: events::Events::default(),
                    password: env!("LAVALINK_PASSWORD").to_string(),
                    user_id: UserId(ctx.cache.current_user_id().0),
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
async fn ready_event(_: LavalinkClient, session_id: String, event: &events::Ready) {
    info!("{:?} -> {:?}", session_id, event);
}
