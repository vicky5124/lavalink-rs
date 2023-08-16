#![allow(clippy::type_complexity)]
#![allow(rustdoc::bare_urls)]

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate serde;

/// The main client, where everything gets done.
pub mod client;
/// Every possible error that the library can return.
pub mod error;
/// The REST API.
pub mod http;
/// Mappings of objects received or sent from or to the API.
pub mod model;
/// A Lavalink server connection.
pub mod node;
/// Player related methods.
pub mod player_context;
/// Re-exports of all the most common types.
pub mod prelude;
