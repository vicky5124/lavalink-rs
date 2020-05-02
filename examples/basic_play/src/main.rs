use std::{
    env,
    sync::Arc
};

use serenity::{
    client::{
        bridge::voice::ClientVoiceManager,
        Context,
    },
    http::Http,
    prelude::*,
    async_trait,
    client::{
        Client,
        EventHandler
    },
    framework::{
        StandardFramework,
        standard::{
            Args, CommandResult,
            macros::{
                command,
                group,
            },
        },
    },
    model::{
        channel::Message,
        gateway::Ready,
        misc::Mentionable
    },
    Result as SerenityResult,
};
use serenity_lavalink::LavalinkClient;

struct VoiceManager;
struct Lavalink;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[only_in("guilds")]
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
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    let mut client = Client::new(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");


    {
        let mut data = client.data.write().await;
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));

        let mut lava_client = LavalinkClient::new();
        lava_client.bot_id = bot_id;
        lava_client.initialize().await?;
        data.insert::<Lavalink>(lava_client);
    }

    let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));

    Ok(())
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = match msg.guild(&ctx.cache).await {
        Some(guild) => guild,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "DMs not supported").await);

            return Ok(());
        }
    };

    let guild_id = guild.read().await.id;

    let channel_id = guild
        .read()
        .await
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(&ctx.http, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().await.
        get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;

    if manager.join(guild_id, connect_to).is_some() {
        check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())).await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel").await);
    }

    Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().await.guild_channel(msg.channel_id) {
        Some(channel) => channel.read().await.guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "DMs not supported").await);

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(&ctx.http, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[min_args(1)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();

    let guild_id = match ctx.cache.read().await.guild_channel(msg.channel_id) {
        Some(channel) => channel.read().await.guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info").await);

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;

    if let Some(handler) = manager.get_mut(guild_id) {
        let data = ctx.data.read().await;
        let lava_client = data.get::<Lavalink>().expect("Expected a lavalink client in TypeMap");

        let query_information = lava_client.auto_search_tracks(&query).await?;

        if query_information.tracks.is_empty() {
            check_msg(msg.channel_id.say(&ctx, "Could not find any video of the search query.").await);
            return Ok(());
        }

        if let Err(why) = lava_client.play(&handler, &query_information.tracks[0]).await {
            eprintln!("{}", why);
            return Ok(());
        };

        check_msg(msg.channel_id.say(&ctx.http, format!("Now playing: {}", query_information.tracks[0].info.title)).await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Use `~join` first, to connect the bot to your current voice channel.").await);
    }

    Ok(())
}

#[command]
async fn ping(context: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&context.http, "Pong!").await);

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        eprintln!("Error sending message: {:?}", why);
    }
}
