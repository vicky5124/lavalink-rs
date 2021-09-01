#[macro_use]
extern crate tracing;

use std::{env, time::Duration};

use lavalink_rs::{gateway::*, model::*, LavalinkClient};
use songbird::SerenityInit;

use poise::{
    async_trait, say_reply, EditTracker, ErrorContext, FrameworkOptions, PrefixFrameworkOptions,
};

use serenity::http::Http;
use serenity::model::prelude::{misc::Mentionable, ApplicationId, Event, Ready, UserId};
use serenity::prelude::{Context as SerenityContext, RawEventHandler};

type Error = Box<dyn std::error::Error + Send + Sync>;
type CommandResult = Result<(), Error>;

type Context<'a> = poise::Context<'a, Data, Error>;
type PrefixContext<'a> = poise::PrefixContext<'a, Data, Error>;

struct Data {
    lavalink: LavalinkClient,
    owner_id: UserId,
}

async fn event_ready(_: SerenityContext, ready: Ready) {
    info!("{} is connected!", ready.user.name);
}

struct Handler;
struct LavalinkHandler;

// NOTE: you cant use the normal event handler with Poise
#[async_trait]
impl RawEventHandler for Handler {
    async fn raw_event(&self, ctx: SerenityContext, event: Event) {
        match event {
            Event::Ready(ready) => event_ready(ctx, ready.ready).await,
            _ => (),
        }
    }
}

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: LavalinkClient, event: TrackStart) {
        info!("Track started!\nGuild: {}", event.guild_id);
    }
    async fn track_finish(&self, _client: LavalinkClient, event: TrackFinish) {
        info!("Track finished!\nGuild: {}", event.guild_id);
    }
}

/// Pong
#[poise::command(slash_command, track_edits)]
async fn ping(ctx: Context<'_>) -> CommandResult {
    say_reply(ctx, "Pong!".to_string()).await?;

    Ok(())
}

/// Register slash commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
#[poise::command(check = "is_owner", hide_in_help)]
async fn register(ctx: PrefixContext<'_>, #[flag] global: bool) -> CommandResult {
    poise::defaults::register_slash_commands(ctx, global).await?;

    Ok(())
}

async fn is_owner(ctx: PrefixContext<'_>) -> Result<bool, Error> {
    Ok(ctx.msg.author.id == ctx.data.owner_id)
}

async fn on_error(error: Error, ctx: ErrorContext<'_, Data, Error>) {
    match ctx {
        ErrorContext::Setup => panic!("Failed to start bot: {:?}", error),
        ErrorContext::Command(ctx) => {
            error!("Error in command `{}`: {:?}", ctx.command().name(), error)
        }
        _ => error!("Other error: {:?}", error),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env::set_var(
        "RUST_LOG",
        "info,poise_basic_queue=trace,poise=debug,serenity=debug,lavalink-rs=debug",
    );
    tracing_subscriber::fmt::init();

    let mut options = FrameworkOptions {
        prefix_options: PrefixFrameworkOptions {
            edit_tracker: Some(EditTracker::for_timespan(Duration::from_secs(500))),
            ..Default::default()
        },
        on_error: |error, ctx| Box::pin(on_error(error, ctx)),
        ..Default::default()
    };

    options.command(ping(), |f| f);
    options.command(register(), |f| f);

    options.command(join(), |f| f);
    options.command(leave(), |f| f);
    options.command(play(), |f| f);
    options.command(skip(), |f| f);
    options.command(now_playing(), |f| f);

    //let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let token = env::var("SLASH_RASH").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    let bot_id = match http.get_current_application_info().await {
        Ok(info) => info.id,
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let lava_client = LavalinkClient::builder(bot_id)
        .set_host("127.0.0.1")
        .set_password(
            env::var("LAVALINK_PASSWORD").unwrap_or_else(|_| "youshallnotpass".to_string()),
        )
        .build(LavalinkHandler)
        .await?;

    let framework = poise::Framework::new(
        ",".to_owned(),          // prefix
        ApplicationId(bot_id.0), // Note that the bot ID is not the same as the application ID in older apps.
        move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    lavalink: lava_client,
                    owner_id: UserId(182891574139682816),
                })
            })
        },
        options,
    );

    framework
        .start(
            serenity::client::ClientBuilder::new_with_http(http)
                .token(token)
                .raw_event_handler(Handler)
                .register_songbird(),
        )
        .await?;

    Ok(())
}

/// Join the voice channel you are on.
#[poise::command(slash_command, track_edits)]
async fn join(ctx: Context<'_>) -> CommandResult {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            say_reply(ctx, "Join a voice channel.".to_string()).await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx.discord()).await.unwrap().clone();

    let (_, handler) = manager.join_gateway(guild_id, connect_to).await;

    match handler {
        Ok(connection_info) => {
            let lava_client = ctx.data().lavalink.clone();
            lava_client.create_session(&connection_info).await?;

            say_reply(ctx, format!("Joined {}", connect_to.mention())).await?;
        }
        Err(why) => say_reply(ctx, format!("Error joining the channel: {}", why)).await?,
    }

    Ok(())
}

/// Leave from a voice channel if connected.
#[poise::command(slash_command, track_edits)]
async fn leave(ctx: Context<'_>) -> CommandResult {
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx.discord()).await.unwrap().clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            say_reply(ctx, format!("Failed: {:?}", e)).await?;
        }

        {
            let lava_client = ctx.data().lavalink.clone();
            lava_client.destroy(guild_id).await?;
        }

        say_reply(ctx, "Left voice channel".to_string()).await?;
    } else {
        say_reply(ctx, "Not in a voice channel".to_string()).await?;
    }

    Ok(())
}

/// Play a song from a URL or a search query.
#[poise::command(slash_command, track_edits)]
async fn play(
    ctx: Context<'_>,
    #[description = "A song URL or YouTube search query"] query: String,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.discord()).await.unwrap().clone();

    if let Some(_handler) = manager.get(guild_id) {
        let lava_client = ctx.data().lavalink.clone();

        let query_information = lava_client.auto_search_tracks(&query).await?;

        if query_information.tracks.is_empty() {
            say_reply(
                ctx,
                "Could not find any video of the search query.".to_string(),
            )
            .await?;
            return Ok(());
        }

        if let Err(why) = &lava_client
            .play(guild_id, query_information.tracks[0].clone())
            // Change this to play() if you want your own custom queue or no queue at all.
            .queue()
            .await
        {
            eprintln!("{}", why);
            return Ok(());
        };

        say_reply(
            ctx,
            format!(
                "Added to queue: {}",
                query_information.tracks[0].info.as_ref().unwrap().title
            ),
        )
        .await?;
    } else {
        say_reply(
            ctx,
            "Use `~join` first, to connect the bot to your current voice channel.".to_string(),
        )
        .await?;
    }

    Ok(())
}

/// Send the currently playing track
#[poise::command(slash_command, track_edits)]
//#[aliases(np)]
async fn now_playing(ctx: Context<'_>) -> CommandResult {
    let lava_client = ctx.data().lavalink.clone();

    if let Some(node) = lava_client.nodes().await.get(&ctx.guild_id().unwrap().0) {
        if let Some(track) = &node.now_playing {
            say_reply(
                ctx,
                format!("Now Playing: {}", track.track.info.as_ref().unwrap().title),
            )
            .await?;
        } else {
            say_reply(ctx, "Nothing is playing at the moment.".to_string()).await?;
        }
    } else {
        say_reply(ctx, "Nothing is playing at the moment.".to_string()).await?;
    }

    Ok(())
}

/// Skip current track
#[poise::command(slash_command, track_edits)]
async fn skip(ctx: Context<'_>) -> CommandResult {
    let lava_client = ctx.data().lavalink.clone();

    if let Some(track) = lava_client.skip(ctx.guild_id().unwrap()).await {
        say_reply(
            ctx,
            format!("Skipped: {}", track.track.info.as_ref().unwrap().title),
        )
        .await?;
    } else {
        say_reply(ctx, "Nothing to skip.".to_string()).await?;
    }

    Ok(())
}
