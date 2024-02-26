pub mod client;
pub mod events;
pub mod http;
pub mod player;
pub mod search;
pub mod track;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

#[pymodule]
pub fn model(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<crate::model::UserId>()?;
    m.add_class::<crate::model::GuildId>()?;
    m.add_class::<crate::model::ChannelId>()?;

    self::client::client(py, m)?;
    m.add_wrapped(wrap_pymodule!(self::events::events))?;
    m.add_wrapped(wrap_pymodule!(self::http::http))?;
    self::player::player(py, m)?;
    m.add_wrapped(wrap_pymodule!(self::search::search))?;
    m.add_wrapped(wrap_pymodule!(self::track::track))?;

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("lavalink_rs.model.client", m.getattr("client")?)?;
    sys_modules.set_item("lavalink_rs.model.events", m.getattr("events")?)?;
    sys_modules.set_item("lavalink_rs.model.http", m.getattr("http")?)?;
    sys_modules.set_item("lavalink_rs.model.player", m.getattr("player")?)?;
    sys_modules.set_item("lavalink_rs.model.search", m.getattr("search")?)?;
    sys_modules.set_item("lavalink_rs.model.track", m.getattr("track")?)?;

    Ok(())
}

#[pymethods]
impl crate::model::UserId {
    #[new]
    fn new_py(user_id: u64) -> Self {
        user_id.into()
    }

    #[getter]
    fn get_inner(&self) -> pyo3::PyResult<u64> {
        Ok(self.0)
    }

    #[setter]
    fn set_inner(&mut self, id: u64) -> pyo3::PyResult<()> {
        self.0 = id;
        Ok(())
    }
}

#[pymethods]
impl crate::model::GuildId {
    #[new]
    fn new_py(user_id: u64) -> Self {
        user_id.into()
    }

    #[getter]
    fn get_inner(&self) -> pyo3::PyResult<u64> {
        Ok(self.0)
    }

    #[setter]
    fn set_inner(&mut self, id: u64) -> pyo3::PyResult<()> {
        self.0 = id;
        Ok(())
    }
}

#[pymethods]
impl crate::model::ChannelId {
    #[new]
    fn new_py(channel_id: u64) -> Self {
        channel_id.into()
    }

    #[getter]
    fn get_inner(&self) -> pyo3::PyResult<u64> {
        Ok(self.0)
    }

    #[setter]
    fn set_inner(&mut self, id: u64) -> pyo3::PyResult<()> {
        self.0 = id;
        Ok(())
    }
}

#[derive(FromPyObject)]
pub enum PyUserId {
    #[pyo3(transparent, annotation = "UserId")]
    UserId(crate::model::UserId),
    #[pyo3(transparent, annotation = "int")]
    Int(u64),
}

impl Into<crate::model::UserId> for PyUserId {
    fn into(self) -> crate::model::UserId {
        match self {
            Self::UserId(x) => x,
            Self::Int(x) => x.into(),
        }
    }
}

#[derive(FromPyObject)]
pub enum PyGuildId {
    #[pyo3(transparent, annotation = "GuildId")]
    GuildId(crate::model::GuildId),
    #[pyo3(transparent, annotation = "int")]
    Int(u64),
}

impl Into<crate::model::GuildId> for PyGuildId {
    fn into(self) -> crate::model::GuildId {
        match self {
            Self::GuildId(x) => x,
            Self::Int(x) => x.into(),
        }
    }
}

#[derive(FromPyObject)]
pub enum PyChannelId {
    #[pyo3(transparent, annotation = "ChannelId")]
    ChannelId(crate::model::ChannelId),
    #[pyo3(transparent, annotation = "int")]
    Int(u64),
}

impl Into<crate::model::ChannelId> for PyChannelId {
    fn into(self) -> crate::model::ChannelId {
        match self {
            Self::ChannelId(x) => x,
            Self::Int(x) => x.into(),
        }
    }
}
