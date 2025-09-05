#[macro_use]
extern crate tracing;

pub mod music_advanced;
pub mod music_basic;
pub mod music_events;

use lavalink_rs::{model::events, prelude::*};

use poise::serenity_prelude as serenity;
use songbird::SerenityInit;

pub struct Data {
    pub lavalink: LavalinkClient,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Test
#[poise::command(prefix_command)]
async fn test(ctx: Context<'_>) -> Result<(), Error> {
    //use std::time::Duration;

    //let guild_id = ctx.guild_id().unwrap();
    let lava_client = ctx.data().lavalink.clone();
    //let lava_client2 = lava_client.clone();
    //let ctx2 = ctx.serenity_context().clone();

    //ctx.say("AAAAAAAAAAA").await?;

    //tokio::spawn(async move {
    //    serenity::model::id::ChannelId::from(702161938012045383)
    //        .say(
    //            &ctx2,
    //            format!(
    //                "{:?}",
    //                lava_client2
    //                    .get_connection_info(124, Duration::from_secs(5))
    //                    .await
    //            ),
    //        )
    //        .await
    //        .unwrap();
    //    serenity::model::id::ChannelId::from(702161938012045383)
    //        .say(
    //            &ctx2,
    //            format!(
    //                "{:?}",
    //                lava_client2
    //                    .get_connection_info(123, Duration::from_secs(5))
    //                    .await
    //            ),
    //        )
    //        .await
    //        .unwrap();
    //});

    //let test_string = "aaa".to_string();

    //lava_client.handle_voice_server_update(123, test_string.clone(), Some(test_string.clone()));
    //lava_client.handle_voice_state_update(123, Some(123), 601749512456896522, test_string.clone());

    //lava_client.handle_voice_server_update(124, test_string.clone(), Some(test_string.clone()));
    //lava_client.handle_voice_state_update(124, Some(123), 601749512456896522, test_string.clone());

    //let player = lava_client.get_player_context(guild_id).unwrap();
    //player.skip()?;
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

    let node = lava_client.get_node_by_index(0).unwrap();
    dbg!(node.http.info().await?);

    ctx.say("all good!").await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "info,lavalink_rs=trace");
    tracing_subscriber::fmt::init();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                music_basic::play(),
                music_basic::join(),
                music_basic::leave(),
                music_advanced::queue(),
                music_advanced::skip(),
                music_advanced::pause(),
                music_advanced::resume(),
                music_advanced::stop(),
                music_advanced::seek(),
                music_advanced::clear(),
                music_advanced::remove(),
                music_advanced::swap(),
                crate::test(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(",".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let events = events::Events {
                    raw: Some(music_events::raw_event),
                    ready: Some(music_events::ready_event),
                    track_start: Some(music_events::track_start),
                    track_end: Some(music_events::track_end),
                    ..Default::default()
                };

                let node_local = NodeBuilder {
                    hostname: "localhost:2333".to_string(),
                    is_ssl: false,
                    events: events::Events::default(),
                    password: env!("LAVALINK_PASSWORD").to_string(),
                    user_id: ctx.cache.current_user().id.into(),
                    session_id: None,
                };

                let client = LavalinkClient::new(
                    events,
                    vec![node_local],
                    NodeDistributionStrategy::round_robin(),
                )
                .await;

                Ok(Data { lavalink: client })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(
        std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"),
        serenity::GatewayIntents::all(),
    )
    .register_songbird()
    .framework(framework)
    .await?;

    client.start().await?;

    Ok(())
}
