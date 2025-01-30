use crate::model::http::*;

use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};

pub fn http(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let http = PyModule::new(py, "http")?;

    http.add_class::<UpdatePlayer>()?;
    http.add_class::<UpdatePlayerTrack>()?;
    http.add_class::<ResumingState>()?;
    http.add_class::<Info>()?;
    http.add_class::<Git>()?;
    http.add_class::<Plugin>()?;
    http.add_class::<Version>()?;

    m.add_submodule(&http)?;

    Ok(())
}

#[pymethods]
impl UpdatePlayer {
    #[new]
    fn new_py() -> UpdatePlayer {
        UpdatePlayer::default()
    }
}

#[apply(crate::python::with_getter_setter)]
#[pymethods]
impl UpdatePlayerTrack {
    getter_setter!((encoded, Option<String>), (identifier, Option<String>),);

    #[new]
    fn new_py() -> UpdatePlayerTrack {
        UpdatePlayerTrack::default()
    }

    #[getter(user_data)]
    fn get_user_data(&self, py: Python<'_>) -> PyObject {
        pythonize(py, &self.user_data).unwrap().into()
    }

    #[setter(user_data)]
    fn set_user_data(&mut self, py: Python<'_>, input: PyObject) {
        self.user_data = depythonize(&input.into_bound(py)).unwrap()
    }
}
