#![allow(clippy::type_complexity)]

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate serde;

pub mod client;
pub mod error;
pub mod http;
pub mod model;
pub mod node;
pub mod player_context;

pub use client::LavalinkClient;
pub use error::LavalinkResult;
pub use model::track::TrackLoadData;
pub use model::GuildId;
pub use model::UserId;
pub use node::NodeBuilder;
pub use player_context::PlayerContext;
pub use player_context::TrackInQueue;
