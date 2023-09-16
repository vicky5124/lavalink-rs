use crate::model::track::*;

use pyo3::prelude::*;

#[pymodule]
pub fn track(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<TrackLoadType>()?;
    //m.add_class::<TrackLoadData>()?;
    //m.add_class::<Track>()?;
    m.add_class::<TrackData>()?;
    m.add_class::<TrackInfo>()?;
    m.add_class::<PlaylistData>()?;
    m.add_class::<PlaylistInfo>()?;
    m.add_class::<TrackError>()?;

    Ok(())
}

#[apply(crate::python::with_getter_setter)]
#[pymethods]
impl TrackData {
    getter_setter!((encoded, String), (info, TrackInfo),);
}

#[apply(crate::python::with_getter_setter)]
#[pymethods]
impl PlaylistData {
    getter_setter!((info, PlaylistInfo), (tracks, Vec<TrackData>),);
}
