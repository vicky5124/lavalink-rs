use std::{
    env,
    sync::Arc,
    collections::HashSet,
    time::Duration,
};

use serenity::client::bridge::voice::ClientVoiceManager;

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
        id::GuildId,
        event::VoiceServerUpdateEvent,
    },
    Result as SerenityResult,
};

use serenity::prelude::*;
use lavalink_rs::{
    LavalinkClient,
    model::*,
    gateway::*,
};

struct VoiceManager;
struct Lavalink;
struct VoiceGuildUpdate;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

impl TypeMapKey for Lavalink {
    type Value = Arc<Mutex<LavalinkClient>>;
}

impl TypeMapKey for VoiceGuildUpdate {
    type Value = Arc<RwLock<HashSet<GuildId>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn voice_server_update(&self, ctx: Context, voice: VoiceServerUpdateEvent) {
        if let Some(guild_id) = voice.guild_id {
            let data = ctx.data.read().await;
            let voice_server_lock = data.get::<VoiceGuildUpdate>().unwrap();
            let mut voice_server = voice_server_lock.write().await;
            voice_server.insert(guild_id);
        }
    }
}


struct LavalinkHandler;

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: Arc<Mutex<LavalinkClient>>, event: TrackStart) {
        println!("Track started!\nGuild: {}", event.guild_id);
    }
    async fn track_finish(&self, _client: Arc<Mutex<LavalinkClient>>, event: TrackFinish) {
        println!("Track finished!\nGuild: {}", event.guild_id);
    }
    async fn stats(&self, _client: Arc<Mutex<LavalinkClient>>, _event: Stats) {
        println!("Stats");
    }
    async fn player_update(&self, _client: Arc<Mutex<LavalinkClient>>, event: PlayerUpdate) {
        println!("Player Update: {:#?}", event);
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
#[commands(join, leave, play, ping)]
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
        .configure(|c| c
                   .prefix("~"))
        .after(after)
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;

        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<VoiceGuildUpdate>(Arc::new(RwLock::new(HashSet::new())));

        let mut lava_client = LavalinkClient::new(bot_id);

        lava_client.set_host("127.0.0.1");

        let lava = lava_client.initialize(LavalinkHandler).await?;
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
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;
    let has_joined = manager.join(guild_id, connect_to).is_some();

    if has_joined {
        drop(manager);

        loop {
            let data = ctx.data.read().await;
            let vgu_lock = data.get::<VoiceGuildUpdate>().unwrap();
            let mut vgu = vgu_lock.write().await;
            if !vgu.contains(&guild_id) {
                tokio::time::delay_for(Duration::from_millis(500)).await;
            } else {
                vgu.remove(&guild_id);
                break;
            }
        }

        let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
        let manager = manager_lock.lock().await;

        let mut data = ctx.data.write().await;
        let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        let handler = manager.get(guild_id).unwrap();
        lava_client_lock.lock().await.create_session(guild_id, &handler).await?;

        check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())).await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel").await);
    }

    Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = ctx.cache.guild_channel_field(msg.channel_id, |channel| channel.guild_id).await.unwrap();

    let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);

        let mut data = ctx.data.write().await;
        let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        lava_client_lock.lock().await.destroy(guild_id).await?;

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

    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;

    if let Some(_handler) = manager.get_mut(guild_id) {
        let mut data = ctx.data.write().await;
        let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
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

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
