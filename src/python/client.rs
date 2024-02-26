use crate::model::events::Events;
use crate::model::http::UpdatePlayer;
use crate::model::player::ConnectionInfo;
use crate::prelude::PlayerContext;

use futures::future::BoxFuture;
use parking_lot::RwLock;
use pyo3::prelude::*;
use pyo3::types::PyList;

#[pymodule]
pub fn client(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<crate::client::LavalinkClient>()?;

    Ok(())
}

fn raw_event(
    _: crate::client::LavalinkClient,
    session_id: String,
    event: &serde_json::Value,
) -> BoxFuture<()> {
    Box::pin(async move {
        debug!("{:?} -> {:?}", session_id, event);
    })
}

#[pymethods]
impl crate::client::LavalinkClient {
    #[pyo3(name = "new")]
    #[staticmethod]
    fn new_py<'a>(
        py: Python<'a>,
        events: PyObject,
        nodes: Vec<crate::node::NodeBuilder>,
        strategy: super::model::client::NodeDistributionStrategyPy,
        user_data: Option<PyObject>,
    ) -> PyResult<&'a PyAny> {
        let current_loop = pyo3_asyncio::get_running_loop(py)?;
        let loop_ref = PyObject::from(current_loop);

        let event_handler = crate::python::event::EventHandler {
            inner: events,
            current_loop: loop_ref,
        };

        let events = Events {
            raw: Some(raw_event),
            event_handler: Some(event_handler),
            ..Default::default()
        };

        pyo3_asyncio::tokio::future_into_py_with_locals(
            py,
            pyo3_asyncio::tokio::get_current_locals(py)?,
            async move {
                if let Some(data) = user_data {
                    Ok(crate::client::LavalinkClient::new_with_data(
                        events,
                        nodes,
                        strategy.inner,
                        std::sync::Arc::new(RwLock::new(data)),
                    )
                    .await)
                } else {
                    Ok(crate::client::LavalinkClient::new_with_data(
                        events,
                        nodes,
                        strategy.inner,
                        std::sync::Arc::new(RwLock::new(Python::with_gil(|py| py.None()))),
                    )
                    .await)
                }
            },
        )
    }

    #[pyo3(name = "create_player_context")]
    fn create_player_context_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
        endpoint: String,
        token: String,
        session_id: String,
        user_data: Option<PyObject>,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py_with_locals(
            py,
            pyo3_asyncio::tokio::get_current_locals(py)?,
            async move {
                if let Some(data) = user_data {
                    Ok(client
                        .create_player_context_with_data(
                            guild_id,
                            ConnectionInfo {
                                endpoint,
                                token,
                                session_id,
                            },
                            std::sync::Arc::new(RwLock::new(data)),
                        )
                        .await?)
                } else {
                    Ok(client
                        .create_player_context_with_data(
                            guild_id,
                            ConnectionInfo {
                                endpoint,
                                token,
                                session_id,
                            },
                            std::sync::Arc::new(RwLock::new(Python::with_gil(|py| py.None()))),
                        )
                        .await?)
                }
            },
        )
    }

    #[pyo3(name = "create_player")]
    fn create_player_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
        endpoint: String,
        token: String,
        session_id: String,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let player = client
                .create_player(
                    guild_id,
                    ConnectionInfo {
                        endpoint,
                        token,
                        session_id,
                    },
                )
                .await?;

            Ok(Python::with_gil(|_py| player))
        })
    }

    #[pyo3(name = "get_player_context")]
    fn get_player_context_py<'a>(
        &self,
        guild_id: super::model::PyGuildId,
    ) -> PyResult<Option<PlayerContext>> {
        let player = self.get_player_context(guild_id);

        Ok(player)
    }

    #[pyo3(name = "load_tracks")]
    fn load_tracks_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
        identifier: String,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let tracks = client.load_tracks(guild_id, &identifier).await?;

            use crate::model::track::TrackLoadData::*;

            Python::with_gil(|py| match tracks.data {
                Some(Track(x)) => Ok(x.into_py(py)),
                Some(Playlist(x)) => Ok(x.into_py(py)),
                Some(Search(x)) => {
                    let l = PyList::empty(py);
                    for i in x {
                        l.append(i.into_py(py))?;
                    }

                    Ok(l.into_py(py))
                }
                Some(Error(x)) => Ok(x.into_py(py)),
                None => Ok(py.None()),
            })
        })
    }

    #[pyo3(name = "delete_player")]
    fn delete_player_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.delete_player(guild_id).await?;

            Ok(())
        })
    }

    #[pyo3(name = "delete_all_player_contexts")]
    fn delete_all_player_contexts_py<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            client.delete_all_player_contexts().await?;

            Ok(())
        })
    }

    #[pyo3(name = "update_player")]
    fn update_player_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
        update_player: UpdatePlayer,
        no_replace: bool,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let player = client
                .update_player(guild_id, &update_player, no_replace)
                .await?;

            Ok(player)
        })
    }

    #[pyo3(name = "decode_track")]
    fn decode_track_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
        track: String,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let track = client.decode_track(guild_id, &track).await?;

            Ok(track)
        })
    }

    #[pyo3(name = "decode_tracks")]
    fn decode_tracks_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
        tracks: Vec<String>,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let tracks = client.decode_tracks(guild_id, &tracks).await?;

            Ok(tracks)
        })
    }

    #[pyo3(name = "request_version")]
    fn request_version_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let version = client.request_version(guild_id).await?;

            Ok(version)
        })
    }

    #[pyo3(name = "request_info")]
    fn request_info_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let info = client.request_info(guild_id).await?;

            Ok(info)
        })
    }

    #[pyo3(name = "request_stats")]
    fn request_stats_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let stats = client.request_stats(guild_id).await?;

            Ok(stats)
        })
    }

    #[pyo3(name = "request_player")]
    fn request_player_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let player = client.request_player(guild_id).await?;

            Ok(player)
        })
    }

    #[pyo3(name = "request_all_players")]
    fn request_all_players_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
    ) -> PyResult<&'a PyAny> {
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let players = client.request_all_players(guild_id).await?;

            Ok(players)
        })
    }

    #[getter]
    #[pyo3(name = "data")]
    fn get_data_py<'a>(&self, py: Python<'a>) -> PyResult<PyObject> {
        let client = self.clone();

        let data = client.data::<RwLock<PyObject>>()?.read().clone_ref(py);

        Ok(data)
    }

    #[setter]
    #[pyo3(name = "data")]
    fn set_data_py(&self, user_data: PyObject) -> PyResult<()> {
        let client = self.clone();

        *client.data::<RwLock<PyObject>>()?.write() = user_data;

        Ok(())
    }

    #[pyo3(name = "handle_voice_server_update", signature = (guild_id, token, endpoint))]
    fn handle_voice_server_update_py(
        &self,
        guild_id: super::model::PyGuildId,
        token: String,
        endpoint: Option<String>,
    ) {
        self.handle_voice_server_update(guild_id, token, endpoint);
    }

    #[pyo3(name = "handle_voice_state_update", signature = (guild_id, channel_id, user_id, session_id))]
    fn handle_voice_state_update_py(
        &self,
        guild_id: super::model::PyGuildId,
        channel_id: Option<super::model::PyChannelId>,
        user_id: super::model::PyUserId,
        session_id: String,
    ) {
        self.handle_voice_state_update(guild_id, channel_id, user_id, session_id);
    }

    #[pyo3(name = "get_connection_info")]
    fn get_connection_info_py<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
        timeout: u64,
    ) -> PyResult<&'a PyAny> {
        let timeout = std::time::Duration::from_millis(timeout);
        let client = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let connection_info = client.get_connection_info(guild_id, timeout).await?;

            Ok(connection_info)
        })
    }
}
