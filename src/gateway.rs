use crate::LavalinkClient;
use crate::model::*;

use std::sync::Arc;

use serenity::async_trait;

#[cfg(feature = "tokio-02-marker")]
use tokio_compat as tokio;

use tokio::sync::Mutex;

#[async_trait]
pub trait LavalinkEventHandler {
    /// Periodic event that returns the statistics of the server.
    async fn stats(&self, _client: Arc<Mutex<LavalinkClient>>, _event: Stats) {}
    /// Event that triggers when a player updates.
    async fn player_update(&self, _client: Arc<Mutex<LavalinkClient>>, _event: PlayerUpdate) {}
    /// Event that triggers when a track starts playing.
    async fn track_start(&self, _client: Arc<Mutex<LavalinkClient>>, _event: TrackStart) {}
    /// Event that triggers when a track finishes playing.
    async fn track_finish(&self, _client: Arc<Mutex<LavalinkClient>>, _event: TrackFinish) {}
}

/*

{"playingPlayers":1,"op":"stats","memory":{"reservable":4294967296,"used":513694368,"free":262251872,"allocated":775946240},"frameStats":{"sent":3000,"deficit":0,"nulled":0},"players":2,"cpu":{"cores":8,"systemLoad":0.12922594961493278,"lavalinkLoad":0.0020833333333333333},"uptime":732761629}

{"playingPlayers":0,"op":"stats","memory":{"reservable":4294967296,"used":496493304,"free":344464648,"allocated":840957952},"players":1,"cpu":{"cores":8,"systemLoad":0.25,"lavalinkLoad":0.40552793689939176},"uptime":797552035}


{"op":"playerUpdate","state":{"position":354760,"time":1595819222861},"guildId":"182892283111276544"}


{"op":"event","type":"TrackStartEvent","track":"QAAAsAIATU5JR0hUV0lTSCAtIFRoZSBHcmVhdGVzdCBTaG93IG9uIEVhcnRoICh3aXRoIFJpY2hhcmQgRGF3a2lucykgKE9GRklDSUFMIExJVkUpAAlOaWdodHdpc2gAAAAAABMyEAALcXJNd3hlMnlhNUUAAQAraHR0cHM6Ly93d3cueW91dHViZS5jb20vd2F0Y2g/dj1xck13eGUyeWE1RQAHeW91dHViZQAAAAAAAAAA","guildId":"182892283111276544"}

{"op":"event","reason":"FINISHED","type":"TrackEndEvent","track":"QAAAjAIAKk5pZ2h0d2lzaCAtIFRoZSBJc2xhbmRlciAoTGl2ZSBBdCBUYW1wZXJlKQAIRWRkIEpvc3MAAAAAAAV2cAALWm84bmNLXzVremMAAQAraHR0cHM6Ly93d3cueW91dHViZS5jb20vd2F0Y2g/dj1abzhuY0tfNWt6YwAHeW91dHViZQAAAAAABXKc","guildId":"182892283111276544"}

*/
