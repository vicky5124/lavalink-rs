use pyo3::prelude::*;
use pyo3::types::PyList;
use pythonize::{depythonize, pythonize};

#[pymodule]
pub fn http(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Http>()?;

    Ok(())
}

#[pyclass]
#[derive(Clone)]
pub(crate) struct Http {
    pub(crate) inner: crate::http::Http,
}

#[pymethods]
impl Http {
    #[getter]
    fn get_authority(&self) -> String {
        self.inner.authority.clone()
    }

    #[setter]
    fn set_authority(&mut self, val: String) {
        self.inner.authority = val;
    }

    #[getter]
    fn get_rest_address(&self) -> String {
        self.inner.rest_address.clone()
    }

    #[setter]
    fn set_rest_address(&mut self, val: String) {
        self.inner.rest_address = val;
    }

    #[getter]
    fn get_rest_address_versionless(&self) -> String {
        self.inner.rest_address_versionless.clone()
    }

    #[setter]
    fn set_rest_address_versionless(&mut self, val: String) {
        self.inner.rest_address_versionless = val;
    }

    pub fn request<'a>(
        &self,
        py: Python<'a>,
        method: String,
        uri: String,
        data: PyObject,
    ) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let data = Python::with_gil(|py| {
                depythonize::<Option<serde_json::Value>>(data.downcast_bound(py)?)
            })?;

            let res = http
                .request::<serde_json::Value, _, _>(
                    ::http::Method::from_bytes(method.as_bytes())
                        .map_err(crate::error::LavalinkError::from)?,
                    uri,
                    data.as_ref(),
                )
                .await?;

            Ok(Python::with_gil(|py| {
                PyObject::from(pythonize(py, &res).unwrap())
            }))
        })
    }

    pub fn raw_request<'a>(
        &self,
        py: Python<'a>,
        method: String,
        uri: String,
        data: PyObject,
    ) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let data = Python::with_gil(|py| {
                depythonize::<Option<serde_json::Value>>(data.downcast_bound(py)?)
            })?;

            let res = http
                .raw_request(
                    ::http::Method::from_bytes(method.as_bytes())
                        .map_err(crate::error::LavalinkError::from)?,
                    uri,
                    data.as_ref(),
                )
                .await?;

            Ok(Python::with_gil(|_py| res))
        })
    }

    /// Destroys the player for this guild in this session.
    pub fn delete_player<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
        session_id: String,
    ) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = http.delete_player(guild_id, &session_id).await?;

            Ok(Python::with_gil(|_py| res))
        })
    }

    /// Updates or creates the player for this guild.
    pub fn update_player<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
        session_id: String,
        data: crate::model::http::UpdatePlayer,
        no_replace: bool,
    ) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = http
                .update_player(guild_id, &session_id, &data, no_replace)
                .await?;

            Ok(Python::with_gil(|_py| res))
        })
    }

    /// Updates the session with the resuming state and timeout.
    pub fn set_resuming_state<'a>(
        &self,
        py: Python<'a>,
        session_id: String,
        resuming_state: crate::model::http::ResumingState,
    ) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = http
                .set_resuming_state(&session_id, &resuming_state)
                .await?;

            Ok(Python::with_gil(|_py| res))
        })
    }

    /// Resolves audio tracks for use with the `update_player` endpoint.
    ///
    /// # Parameters
    ///
    /// - `identifier`: A track identifier.
    ///  - Can be a url: "https://youtu.be/watch?v=DrM2lo6B04I"
    ///  - A unique identifier: "DrM2lo6B04I"
    ///  - A search: "ytsearch:Ne Obliviscaris - Forget Not"
    pub fn load_tracks<'a>(
        &self,
        py: Python<'a>,
        identifier: String,
    ) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let tracks = http.load_tracks(&identifier).await?;

            use crate::model::track::TrackLoadData::*;

            Python::with_gil(|py| {
                let track_data: Option<PyObject> = match tracks.data {
                    Some(Track(x)) => Some(x.into_pyobject(py).unwrap().into_any()),
                    Some(Playlist(x)) => Some(x.into_pyobject(py).unwrap().into_any()),
                    Some(Search(x)) => {
                        let l = PyList::empty(py);
                        for i in x {
                            l.append(i.into_pyobject(py).unwrap())?;
                        }

                        Some(l.into_pyobject(py).unwrap().into_any())
                    }
                    Some(Error(x)) => Some(x.into_pyobject(py).unwrap().into_any()),
                    None => None,
                }
                .map(|x| x.into());

                Ok(super::model::track::Track {
                    load_type: tracks.load_type,
                    data: track_data,
                })
            })
        })
    }

    /// Request Lavalink server version.
    pub fn version<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = http.version().await?;

            Ok(Python::with_gil(|_py| res))
        })
    }

    /// Request Lavalink statistics.
    ///
    /// NOTE: The frame stats will never be returned.
    pub fn stats<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = http.stats().await?;

            Ok(Python::with_gil(|_py| res))
        })
    }

    /// Request Lavalink server information.
    pub fn info<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = http.info().await?;

            Ok(Python::with_gil(|_py| res))
        })
    }

    /// Decode a single track into its info.
    ///
    /// # Parameters
    ///
    /// - `track`: base64 encoded track data.
    pub fn decode_track<'a>(&self, py: Python<'a>, track: String) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = http.decode_track(&track).await?;

            Ok(Python::with_gil(|_py| res))
        })
    }

    /// Decode multiple tracks into their info.
    ///
    /// # Parameters
    ///
    /// - `tracks`: base64 encoded tracks.
    pub fn decode_tracks<'a>(
        &self,
        py: Python<'a>,
        tracks: Vec<String>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = http.decode_tracks(&tracks).await?;

            Ok(Python::with_gil(|_py| res))
        })
    }

    /// Returns the player for this guild in this session.
    pub fn get_player<'a>(
        &self,
        py: Python<'a>,
        guild_id: super::model::PyGuildId,
        session_id: String,
    ) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = http.get_player(guild_id, &session_id).await?;

            Ok(Python::with_gil(|_py| res))
        })
    }

    /// Returns a list of players in this specific session.
    pub fn get_players<'a>(
        &self,
        py: Python<'a>,
        session_id: String,
    ) -> PyResult<Bound<'a, PyAny>> {
        let http = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = http.get_players(&session_id).await?;

            Ok(Python::with_gil(|_py| res))
        })
    }
}
