#![allow(stable_features)]
#![feature(or_patterns)] // stable on 1.53.0

#[macro_use]
extern crate tracing;

use std::{env, error::Error, sync::Arc};

use futures::StreamExt;

use lavalink_rs::{async_trait, gateway::*, model::*, LavalinkClient};
use songbird::Songbird;

use twilight_command_parser::{Arguments, Command, CommandParserConfig, Parser};
use twilight_gateway::{Cluster, Event, Intents};
use twilight_http::Client as HttpClient;
use twilight_model::channel::Message;

const COMMAND_PREFIX: &'static str = ",";

type ServiceResult = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(Clone)]
struct Context {
    cluster: Cluster,
    http: HttpClient,
    songbird: Arc<Songbird>,
    lavalink: LavalinkClient,
}

struct LavalinkHandler;

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: LavalinkClient, event: TrackStart) {
        info!("Track started!\nGuild: {}", event.guild_id);
    }
    async fn track_finish(&self, _client: LavalinkClient, event: TrackFinish) {
        info!("Track finished!\nGuild: {}", event.guild_id);
    }
}

#[tokio::main]
async fn main() -> ServiceResult {
    env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();

    info!("Tracing event logger initialized.");

    let context = {
        let token = env::var("DISCORD_TOKEN")?;

        let http = HttpClient::new(&token);
        let cluster =
            Cluster::new(token, Intents::GUILD_MESSAGES | Intents::GUILD_VOICE_STATES).await?;

        let bot_id = http.current_user().await?.id;
        let shard_count = cluster.shards().len();

        info!(
            "Logging in with user ID {} on {} shards",
            bot_id, shard_count
        );

        let songbird = Songbird::twilight(cluster.clone(), shard_count as u64, bot_id);

        let lavalink = LavalinkClient::builder(bot_id)
            .set_host("127.0.0.1")
            .set_password("youshallnotpass")
            .set_shard_count(shard_count as u64)
            .build(LavalinkHandler)
            .await?;

        cluster.up().await;

        Context {
            cluster,
            http,
            songbird,
            lavalink,
        }
    };

    let mut commands_config = CommandParserConfig::new();

    commands_config.add_prefix(COMMAND_PREFIX);
    commands_config.add_command("ping", true);
    commands_config.add_command("join", true);
    commands_config.add_command("leave", true);
    commands_config.add_command("play", true);
    commands_config.add_command("now_playing", true);
    commands_config.add_command("np", true);
    commands_config.add_command("skip", true);

    let parser = Parser::new(commands_config);

    let mut events = context.cluster.events();

    while let Some(event) = events.next().await {
        context.songbird.process(&event.1).await;

        if let Event::MessageCreate(ref msg) = event.1 {
            if msg.guild_id.is_none() || !msg.content.starts_with(COMMAND_PREFIX) {
                continue;
            }

            // Yeah, i know i shouldn't be cloning this much, but it's just an example, give some
            // slack. You are using twilight instead of serenity, so you can do better than this :P
            let parser_clone = parser.clone();
            let context_clone = context.clone();
            let message = msg.0.clone();

            tokio::spawn(async move {
                if let Err(why) = parse_command(parser_clone, context_clone, message).await {
                    error!("Error running command: {}", why);
                }
            });
        }

        if let Event::Ready(_) = event.1 {
            info!("Bot is ready!");
        }
    }

    Ok(())
}

async fn parse_command(parser: Parser<'_>, ctx: Context, msg: Message) -> ServiceResult {
    match parser.parse(&msg.content) {
        Some(Command { name: "ping", .. }) => {
            ping(ctx, &msg).await?;
        }
        Some(Command {
            name: "join",
            arguments,
            ..
        }) => {
            join(ctx, &msg, arguments).await?;
        }
        Some(Command { name: "leave", .. }) => {
            leave(ctx, &msg).await?;
        }
        Some(Command {
            name: "play",
            arguments,
            ..
        }) => {
            play(ctx, &msg, arguments).await?;
        }
        Some(Command {
            name: "now_playing" | "np",
            ..
        }) => {
            now_playing(ctx, &msg).await?;
        }
        Some(Command { name: "skip", .. }) => {
            skip(ctx, &msg).await?;
        }

        _ => (),
    }

    Ok(())
}

async fn ping(ctx: Context, msg: &Message) -> ServiceResult {
    ctx.http
        .create_message(msg.channel_id)
        .content("Pong!")?
        .await?;

    Ok(())
}

async fn join(ctx: Context, msg: &Message, args: Arguments<'_>) -> ServiceResult {
    let args = args.as_str();

    let vc_id = match args.parse::<u64>() {
        Ok(x) => x,
        Err(why) => {
            ctx.http
                .create_message(msg.channel_id)
                .content(format!(
                    "Please provide a valid voice channel ID\n```{}```",
                    why
                ))?
                .await?;

            return Ok(());
        }
    };

    let guild_id = msg.guild_id.unwrap();

    let (_, handle) = ctx.songbird.join_gateway(guild_id, vc_id).await;

    let content = match handle {
        Ok(connection_info) => {
            ctx.lavalink.create_session(&connection_info).await?;

            format!("Joined <#{}>!", vc_id)
        }
        Err(e) => format!("Failed to join <#{}>\n```{:?}```", vc_id, e),
    };

    ctx.http
        .create_message(msg.channel_id)
        .content(content)?
        .await?;

    Ok(())
}

async fn leave(ctx: Context, msg: &Message) -> ServiceResult {
    let guild_id = msg.guild_id.unwrap();

    let has_handler = ctx.songbird.get(guild_id).is_some();

    if has_handler {
        if let Err(why) = ctx.songbird.remove(guild_id).await {
            error!("Failed to leave channel: {}", why);

            ctx.http
                .create_message(msg.channel_id)
                .content("Failed to leave the channel.")?
                .await?;
        }

        ctx.lavalink.destroy(guild_id).await?;

        ctx.http
            .create_message(msg.channel_id)
            .content("Left voice channel.")?
            .await?;
    } else {
        ctx.http
            .create_message(msg.channel_id)
            .content("Not in a voice channel.")?
            .await?;
    }

    Ok(())
}

async fn play(ctx: Context, msg: &Message, args: Arguments<'_>) -> ServiceResult {
    let query = args.as_str();

    if query.is_empty() {
        ctx.http
            .create_message(msg.channel_id)
            .content("Please, specify something to play.")?
            .await?;

        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();

    if let Some(_handler) = ctx.songbird.get(guild_id) {
        let query_information = ctx.lavalink.auto_search_tracks(&query).await?;

        if query_information.tracks.is_empty() {
            ctx.http
                .create_message(msg.channel_id)
                .content("Could not find any video of the search query.")?
                .await?;

            return Ok(());
        }

        if let Err(why) = &ctx
            .lavalink
            .play(guild_id, query_information.tracks[0].clone())
            // Change this to play() if you want your own custom queue or no queue at all.
            .queue()
            .await
        {
            error!("Error playing: {}", why);

            ctx.http
                .create_message(msg.channel_id)
                .content("Could not play the track.")?
                .await?;

            return Ok(());
        };

        ctx.http
            .create_message(msg.channel_id)
            .content(format!(
                "Added to queue: {}",
                query_information.tracks[0].info.as_ref().unwrap().title
            ))?
            .await?;
    } else {
        ctx.http
            .create_message(msg.channel_id)
            .content(format!(
                "Make the bot join a voice channel first using `{}join <channel_id>` first",
                COMMAND_PREFIX
            ))?
            .await?;
    }

    Ok(())
}

async fn now_playing(ctx: Context, msg: &Message) -> ServiceResult {
    if let Some(node) = ctx.lavalink.nodes().await.get(&msg.guild_id.unwrap().0) {
        if let Some(track) = &node.now_playing {
            ctx.http
                .create_message(msg.channel_id)
                .content(format!(
                    "Now Playing: {}",
                    track.track.info.as_ref().unwrap().title
                ))?
                .await?;
        } else {
            ctx.http
                .create_message(msg.channel_id)
                .content("Nothing is playing at the moment.")?
                .await?;
        }
    } else {
        ctx.http
            .create_message(msg.channel_id)
            .content("Nothing is playing at the moment.")?
            .await?;
    }

    Ok(())
}

async fn skip(ctx: Context, msg: &Message) -> ServiceResult {
    if let Some(track) = ctx.lavalink.skip(msg.guild_id.unwrap()).await {
        ctx.http
            .create_message(msg.channel_id)
            .content(format!(
                "Skipped: {}",
                track.track.info.as_ref().unwrap().title
            ))?
            .await?;
    } else {
        ctx.http
            .create_message(msg.channel_id)
            .content("Nothing to skip.")?
            .await?;
    }

    Ok(())
}
