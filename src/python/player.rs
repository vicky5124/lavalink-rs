use std::time::Duration;

use crate::{
    model::{
        http::UpdatePlayer,
        player::{Filters, Player},
        track::TrackData,
    },
    player_context::{QueueMessage, TrackInQueue},
};

use parking_lot::RwLock;
use pyo3::prelude::*;

#[pymodule]
pub fn player(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<crate::player_context::PlayerContext>()?;
    m.add_class::<crate::player_context::TrackInQueue>()?;

    Ok(())
}

#[pymethods]
impl crate::player_context::PlayerContext {
    fn set_queue_push_to_back<'a>(&self, track: PyTrackInQueue) -> PyResult<()> {
        self.set_queue(QueueMessage::PushToBack(track.into()))?;
        Ok(())
    }
    fn set_queue_push_to_front<'a>(&self, track: PyTrackInQueue) -> PyResult<()> {
        self.set_queue(QueueMessage::PushToFront(track.into()))?;
        Ok(())
    }
    fn set_queue_insert<'a>(&self, position: usize, track: PyTrackInQueue) -> PyResult<()> {
        self.set_queue(QueueMessage::Insert(position, track.into()))?;
        Ok(())
    }
    fn set_queue_remove<'a>(&self, position: usize) -> PyResult<()> {
        self.set_queue(QueueMessage::Remove(position))?;
        Ok(())
    }
    fn set_queue_clear<'a>(&self) -> PyResult<()> {
        self.set_queue(QueueMessage::Clear)?;
        Ok(())
    }
    fn set_queue_replace<'a>(&self, tracks: Vec<PyTrackInQueue>) -> PyResult<()> {
        self.set_queue(QueueMessage::Replace(
            tracks.into_iter().map(TrackInQueue::from).collect(),
        ))?;
        Ok(())
    }
    fn set_queue_append<'a>(&self, tracks: Vec<PyTrackInQueue>) -> PyResult<()> {
        self.set_queue(QueueMessage::Append(
            tracks.into_iter().map(TrackInQueue::from).collect(),
        ))?;
        Ok(())
    }

    #[pyo3(name = "get_queue")]
    fn get_queue_py<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let player = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let queue: Vec<_> = player.get_queue().await?.into();

            Ok(Python::with_gil(|_py| queue))
        })
    }

    #[pyo3(name = "get_player")]
    fn get_player_py<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let player = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let player_inner = player.get_player().await?;

            Ok(Python::with_gil(|_py| player_inner))
        })
    }

    #[pyo3(name = "update_player")]
    fn update_player_py<'a>(
        &self,
        py: Python<'a>,
        update_player: UpdatePlayer,
        no_replace: bool,
    ) -> PyResult<&'a PyAny> {
        let player = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let player_inner = player.update_player(&update_player, no_replace).await?;

            Ok(Python::with_gil(|_py| player_inner))
        })
    }

    #[pyo3(name = "play")]
    fn play_py<'a>(
        &self,
        py: Python<'a>,
        track: crate::model::track::TrackData,
    ) -> PyResult<&'a PyAny> {
        let player = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            player.play(&track).await?;

            Ok(Python::with_gil(|py| py.None()))
        })
    }

    #[pyo3(name = "play_now")]
    fn play_now_py<'a>(
        &self,
        py: Python<'a>,
        track: crate::model::track::TrackData,
    ) -> PyResult<&'a PyAny> {
        let player = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            player.play_now(&track).await?;

            Ok(Python::with_gil(|py| py.None()))
        })
    }

    #[pyo3(name = "stop_now")]
    fn stop_now_py<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        let player = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let player_inner = player.stop_now().await?;

            Ok(Python::with_gil(|_py| player_inner))
        })
    }

    #[pyo3(name = "set_pause")]
    fn set_pause_py<'a>(&self, py: Python<'a>, pause: bool) -> PyResult<&'a PyAny> {
        let player = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let player_inner = player.set_pause(pause).await?;

            Ok(Python::with_gil(|_py| player_inner))
        })
    }

    #[pyo3(name = "set_volume")]
    fn set_volume_py<'a>(&self, py: Python<'a>, volume: u16) -> PyResult<&'a PyAny> {
        let player = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let player_inner = player.set_volume(volume).await?;

            Ok(Python::with_gil(|_py| player_inner))
        })
    }

    #[pyo3(name = "set_filters")]
    fn set_filters_py<'a>(&self, py: Python<'a>, filters: Filters) -> PyResult<&'a PyAny> {
        let player = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let player_inner = player.set_filters(filters).await?;

            Ok(Python::with_gil(|_py| player_inner))
        })
    }

    #[pyo3(name = "set_position_ms")]
    fn set_position_ms_py<'a>(&self, py: Python<'a>, position: u64) -> PyResult<&'a PyAny> {
        let player = self.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let player_inner = player.set_position(Duration::from_millis(position)).await?;

            Ok(Python::with_gil(|_py| player_inner))
        })
    }

    #[pyo3(name = "queue")]
    fn queue_py<'a>(&self, track: PyTrackInQueue) -> PyResult<()> {
        self.queue(track)?;
        Ok(())
    }

    #[pyo3(name = "close")]
    fn close_py<'a>(&self) -> PyResult<()> {
        self.clone().close()?;
        Ok(())
    }

    #[pyo3(name = "skip")]
    fn skip_py<'a>(&self) -> PyResult<()> {
        self.skip()?;
        Ok(())
    }

    #[pyo3(name = "finish")]
    fn finish_py<'a>(&self, should_continue: bool) -> PyResult<()> {
        self.finish(should_continue)?;
        Ok(())
    }

    #[pyo3(name = "update_player_data")]
    fn update_player_data_py<'a>(&self, player: Player) -> PyResult<()> {
        self.update_player_data(player)?;
        Ok(())
    }

    #[getter]
    #[pyo3(name = "data")]
    fn get_data_py<'a>(&self, py: Python<'a>) -> PyResult<PyObject> {
        let player = self.clone();

        let data = player.data::<RwLock<PyObject>>()?.read().clone_ref(py);

        Ok(data)
    }

    #[setter]
    #[pyo3(name = "data")]
    fn set_data_py(&self, user_data: PyObject) -> PyResult<()> {
        let player = self.clone();

        *player.data::<RwLock<PyObject>>()?.write() = user_data;

        Ok(())
    }
}

#[apply(crate::python::with_getter_setter)]
#[pymethods]
impl crate::player_context::TrackInQueue {
    getter_setter!(
        (track, crate::model::track::TrackData),
        (volume, Option<u16>),
        (filters, Option<crate::model::player::Filters>),
    );

    #[getter]
    fn get_start_time_ms(&self) -> Option<u128> {
        self.start_time.map(|ms| ms.as_millis())
    }

    #[setter]
    fn set_start_time_ms(&mut self, ms: Option<u64>) -> pyo3::PyResult<()> {
        self.start_time = ms.map(|ms| Duration::from_millis(ms));
        Ok(())
    }

    #[getter]
    fn get_end_time_ms(&self) -> Option<u128> {
        self.end_time.map(|ms| ms.as_millis())
    }

    #[setter]
    fn set_end_time_ms(&mut self, ms: Option<u64>) -> pyo3::PyResult<()> {
        self.end_time = ms.map(|ms| Duration::from_millis(ms));
        Ok(())
    }
}

#[derive(FromPyObject)]
pub enum PyTrackInQueue {
    #[pyo3(transparent, annotation = "TrackInQueue")]
    TrackInQueue(TrackInQueue),
    #[pyo3(transparent, annotation = "TrackData")]
    TrackData(TrackData),
}

impl From<PyTrackInQueue> for TrackInQueue {
    fn from(value: PyTrackInQueue) -> Self {
        match value {
            PyTrackInQueue::TrackInQueue(x) => x,
            PyTrackInQueue::TrackData(x) => x.into(),
        }
    }
}
