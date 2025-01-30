use crate::model::track::*;

use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};

#[pymodule]
pub fn track(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TrackLoadType>()?;
    //m.add_class::<TrackLoadData>()?;
    m.add_class::<Track>()?;
    m.add_class::<TrackData>()?;
    m.add_class::<TrackInfo>()?;
    m.add_class::<PlaylistData>()?;
    m.add_class::<PlaylistInfo>()?;
    m.add_class::<TrackError>()?;

    Ok(())
}

#[pyclass(get_all, set_all)]
pub(crate) struct Track {
    pub(crate) load_type: TrackLoadType,
    pub(crate) data: Option<PyObject>,
}

#[apply(crate::python::with_getter_setter)]
#[pymethods]
impl TrackData {
    getter_setter!((encoded, String), (info, TrackInfo),);

    #[getter(plugin_info)]
    fn get_plugin_info(&self, py: Python<'_>) -> PyObject {
        pythonize(py, &self.plugin_info).unwrap().into()
    }

    #[setter(plugin_info)]
    fn set_plugin_info(&mut self, py: Python<'_>, input: PyObject) {
        self.plugin_info = depythonize(&input.into_bound(py)).unwrap()
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

#[apply(crate::python::with_getter_setter)]
#[pymethods]
impl PlaylistData {
    getter_setter!((info, PlaylistInfo), (tracks, Vec<TrackData>),);

    #[getter(plugin_info)]
    fn get_plugin_info(&self, py: Python<'_>) -> PyObject {
        pythonize(py, &self.plugin_info).unwrap().into()
    }

    #[setter(plugin_info)]
    fn set_plugin_info(&mut self, py: Python<'_>, input: PyObject) {
        self.plugin_info = depythonize(&input.into_bound(py)).unwrap()
    }
}
