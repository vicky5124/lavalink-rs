use crate::model::search::*;

use pyo3::prelude::*;

#[pymodule]
pub fn search(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<SpotifyRecommendedParameters>()?;
    m.add_class::<FloweryTTSParameters>()?;

    Ok(())
}
