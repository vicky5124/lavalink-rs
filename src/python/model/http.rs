use crate::model::http::*;

use pyo3::prelude::*;

pub fn http(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    let http = PyModule::new(py, "http")?;

    http.add_class::<UpdatePlayer>()?;
    http.add_class::<UpdatePlayerTrack>()?;
    http.add_class::<ResumingState>()?;
    http.add_class::<Info>()?;
    http.add_class::<Git>()?;
    http.add_class::<Plugin>()?;
    http.add_class::<Version>()?;

    m.add_submodule(http)?;

    Ok(())
}

#[pymethods]
impl UpdatePlayer {
    #[new]
    fn new_py() -> UpdatePlayer {
        UpdatePlayer::default()
    }
}

#[pymethods]
impl UpdatePlayerTrack {
    #[new]
    fn new_py() -> UpdatePlayerTrack {
        UpdatePlayerTrack::default()
    }
}
