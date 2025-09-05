use lavalink_rs::{hook, model::events, prelude::*};
use poise::serenity_prelude::{model::id::ChannelId, Http};

// The #[hook] macro transforms:
// ```rs
// #[hook]
// async fn foo(a: A) -> T {
//     ...
// }
// ```
// into
// ```rs
// fn foo<'a>(a: A) -> Pin<Box<dyn Future<Output = T> + Send + 'a>> {
//     Box::pin(async move {
//         ...
//     })
// }
// ```
//
// This allows the asynchronous function to be stored in a structure.

#[hook]
pub async fn raw_event(_: LavalinkClient, session_id: String, event: &serde_json::Value) {
    if event["op"].as_str() == Some("event") || event["op"].as_str() == Some("playerUpdate") {
        info!("{:?} -> {:?}", session_id, event);
    }
}

#[hook]
pub async fn ready_event(client: LavalinkClient, session_id: String, event: &events::Ready) {
    client.delete_all_player_contexts().await.unwrap();
    info!("{:?} -> {:?}", session_id, event);
}

#[hook]
pub async fn track_start(client: LavalinkClient, _session_id: String, event: &events::TrackStart) {
    let player_context = client.get_player_context(event.guild_id).unwrap();
    let data = player_context
        .data::<(ChannelId, std::sync::Arc<Http>)>()
        .unwrap();
    let (channel_id, http) = (&data.0, &data.1);

    let msg = {
        let track = &event.track;

        if let Some(uri) = &track.info.uri {
            format!(
                "Now playing: [{} - {}](<{}>) | Requested by <@!{}>",
                track.info.author,
                track.info.title,
                uri,
                track.user_data.clone().unwrap()["requester_id"]
            )
        } else {
            format!(
                "Now playing: {} - {} | Requested by <@!{}>",
                track.info.author,
                track.info.title,
                track.user_data.clone().unwrap()["requester_id"]
            )
        }
    };

    let _ = channel_id.say(http, msg).await;
}

#[hook]
pub async fn track_end(client: LavalinkClient, _session_id: String, event: &events::TrackEnd) {
    let player_context = client.get_player_context(event.guild_id).unwrap();
    debug!("Songs left in queue: {:?}", player_context.get_queue().get_count().await);
}
