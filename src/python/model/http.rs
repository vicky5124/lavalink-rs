use crate::model::http::*;

use pyo3::prelude::*;

#[pymodule]
pub fn http(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<UpdatePlayer>()?;
    m.add_class::<ResumingState>()?;
    m.add_class::<Info>()?;
    m.add_class::<Git>()?;
    m.add_class::<Plugin>()?;
    m.add_class::<Version>()?;

    Ok(())
}

#[pymethods]
impl UpdatePlayer {
    #[new]
    fn new_py() -> UpdatePlayer {
        UpdatePlayer::default()
    }
}
