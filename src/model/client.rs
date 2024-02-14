use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use super::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub(crate) enum ClientMessage {
    GetConnectionInfo(
        GuildId,
        std::time::Duration,
        oneshot::Sender<Result<player::ConnectionInfo, tokio::time::error::Elapsed>>,
    ),
    ServerUpdate(GuildId, String, Option<String>), // guild_id, token, endpoint
    StateUpdate(GuildId, Option<ChannelId>, UserId, String), // guild_id, channel_id, user_id, session_id
}

#[derive(Debug, Default, Clone)]
pub enum NodeDistributionStrategy {
    #[default]
    Sharded,
    RoundRobin(Arc<AtomicUsize>),
    MainFallback,
    LowestLoad,
    HighestFreeMemory,
    //ByRegion(...),
    Custom(fn(&'_ crate::client::LavalinkClient, GuildId) -> BoxFuture<Arc<crate::node::Node>>),
    #[cfg(feature = "python")]
    CustomPython(PyObject),
}

impl NodeDistributionStrategy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sharded() -> Self {
        Self::Sharded
    }

    pub fn round_robin() -> Self {
        Self::RoundRobin(std::sync::Arc::new(AtomicUsize::new(0)))
    }

    pub fn main_fallback() -> Self {
        Self::MainFallback
    }

    pub fn lowest_load() -> Self {
        Self::LowestLoad
    }

    pub fn highest_free_memory() -> Self {
        Self::HighestFreeMemory
    }

    pub fn custom(
        func: fn(&'_ crate::client::LavalinkClient, GuildId) -> BoxFuture<Arc<crate::node::Node>>,
    ) -> NodeDistributionStrategy {
        NodeDistributionStrategy::Custom(func)
    }
}
