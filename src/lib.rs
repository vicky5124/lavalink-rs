#![allow(clippy::type_complexity)]
#![allow(rustdoc::bare_urls)]

#[cfg(not(feature = "python"))]
#[macro_use]
extern crate tracing;

#[cfg(feature = "python")]
#[macro_use]
extern crate log;

#[cfg(feature = "python")]
#[macro_use]
extern crate macro_rules_attribute;

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
/// Macros that abstract annoying stuff.
#[cfg(feature = "macros")]
pub mod macros {
    /// A macro that transforms `async` functions (and closures) into plain functions, whose return
    /// type is a boxed [`Future`].
    ///
    /// [`Future`]: std::future::Future
    pub use macros_dep::hook;
}

#[cfg(feature = "macros")]
/// A macro that transforms `async` functions (and closures) into plain functions, whose return
/// type is a boxed [`Future`].
///
/// [`Future`]: std::future::Future
pub use macros::hook;

#[cfg(feature = "python")]
use pyo3::{prelude::*, types::PyDict, wrap_pymodule};

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
#[pymodule]
fn lavalink_rs(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    let handle = pyo3_log::Logger::new(py, pyo3_log::Caching::LoggersAndLevels)?
        .filter(log::LevelFilter::Trace)
        .install()
        .expect("Someone installed a logger before lavalink_rs.");

    // Some time in the future when logging changes, reset the caches:
    handle.reset();

    m.add_class::<client::LavalinkClient>()?;
    m.add_class::<player_context::PlayerContext>()?;

    m.add_class::<python::event::EventHandler>()?;
    m.add_class::<node::NodeBuilder>()?;
    m.add_class::<python::model::client::NodeDistributionStrategyPy>()?;
    m.add_class::<player_context::TrackInQueue>()?;

    m.add_class::<model::UserId>()?;
    m.add_class::<model::ChannelId>()?;
    m.add_class::<model::GuildId>()?;

    m.add_wrapped(wrap_pymodule!(python::model::model))?;

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("lavalink_rs.model", m.getattr("model")?)?;

    Ok(())
}
