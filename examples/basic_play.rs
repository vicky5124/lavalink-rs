use std::{
    env,
    sync::Arc,
};

use serenity::{
    client::Context,
    prelude::Mutex,
};

use serenity::{
    async_trait,
    client::{
        Client,
        EventHandler
    },
    http::Http,
    framework::{
        StandardFramework,
        standard::{
            Args,
            CommandResult,
            macros::{
                command,
                group,
                hook,
            },
        },
    },
    model::{
        channel::Message,
        gateway::Ready,
        misc::Mentionable,
    },
    Result as SerenityResult,
};

use serenity::prelude::*;
use songbird::SerenityInit;
use lavalink_rs::{
    LavalinkClient,
    model::*,
    gateway::*,
};

struct Lavalink;

impl TypeMapKey for Lavalink {
    type Value = Arc<Mutex<LavalinkClient>>;
}

struct Handler;
struct LavalinkHandler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: Arc<Mutex<LavalinkClient>>, event: TrackStart) {
        println!("Track started!\nGuild: {}", event.guild_id);
    }
    async fn track_finish(&self, _client: Arc<Mutex<LavalinkClient>>, event: TrackFinish) {
        println!("Track finished!\nGuild: {}", event.guild_id);
    }
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Err(why) => println!("Command '{}' returned error {:?} => {}", command_name, why, why),
        _ => (),
    }
}

#[group]
#[only_in(guilds)]
#[commands(join, leave, play, now_playing, skip, ping)]
struct General;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    let bot_id = match http.get_current_application_info().await {
        Ok(info) => info.id,
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .after(after)
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    let mut lava_client = LavalinkClient::new(bot_id);
    lava_client.set_host("127.0.0.1");

    let lava = lava_client.initialize(LavalinkHandler).await?;

    {
        let mut data = client.data.write().await;
        data.insert::<Lavalink>(lava);
    }

    let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));

    Ok(())
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Join a voice channel.").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.unwrap().clone();

    let (_, handler) = manager.join_gateway(guild_id, connect_to).await;

    match handler {
        Ok(connection_info) => {
            let mut data = ctx.data.write().await;
            let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
            lava_client_lock.lock().await.create_session(guild_id, &connection_info.recv_async().await?).await?;

            check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())).await);
        }
        Err(why) => check_msg(msg.channel_id.say(&ctx.http, format!("Error joining the channel: {}", why)).await),
    }

    Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await.unwrap().clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        {
            let mut data = ctx.data.write().await;
            let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
            lava_client_lock.lock().await.destroy(guild_id).await?;
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
async fn ping(context: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&context.http, "Pong!").await);

    Ok(())
}

#[command]
#[min_args(1)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();

    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info").await);

            return Ok(());
        },
    };

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(_handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        let lava_client_lock = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        let mut lava_client = lava_client_lock.lock().await;

        let query_information = lava_client.auto_search_tracks(&query).await?;

        if query_information.tracks.is_empty() {
            check_msg(msg.channel_id.say(&ctx, "Could not find any video of the search query.").await);
            return Ok(());
        }

        if let Some(ref mut socket) = lava_client.socket_write {
            if let Err(why) = LavalinkClient::play(guild_id, query_information.tracks[0].clone()).start(socket).await {
                eprintln!("{}", why);
                return Ok(());
            };
            check_msg(msg.channel_id.say(&ctx.http, format!("Now playing: {}", query_information.tracks[0].info.as_ref().unwrap().title)).await);
        } else {

            eprintln!("No websocket found");
        }
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Use `~join` first, to connect the bot to your current voice channel.").await);
    }

    Ok(())
}

#[command]
#[aliases(np)]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
    let lava_client = lava_client_lock.lock().await;

    if let Some(node) = lava_client.nodes.get(&msg.guild_id.unwrap().0) {
        if let Some(track) = &node.now_playing {
            check_msg(msg.channel_id.say(&ctx.http, format!("Now Playing: {}", track.track.info.as_ref().unwrap().title)).await);
        } else {
            check_msg(msg.channel_id.say(&ctx.http, "Nothing is playing at the moment.").await);
        }
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Nothing is playing at the moment.").await);
    }

    Ok(())
}

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");

    if let Some(track) = lava_client_lock.lock().await.skip(msg.guild_id.unwrap()).await {
        check_msg(msg.channel_id.say(ctx, format!("Skipped: {}", track.track.info.as_ref().unwrap().title)).await);
    } else {
        check_msg(msg.channel_id.say(ctx, "Nothing to skip.").await);
    }

    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
