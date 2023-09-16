use crate::model::events::*;

use pyo3::prelude::*;

#[pymodule]
pub fn events(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Ready>()?;
    m.add_class::<PlayerUpdate>()?;
    m.add_class::<Stats>()?;
    m.add_class::<Cpu>()?;
    m.add_class::<Memory>()?;
    m.add_class::<FrameStats>()?;
    m.add_class::<TrackStart>()?;
    m.add_class::<TrackEnd>()?;
    m.add_class::<TrackEndReason>()?;
    m.add_class::<TrackException>()?;
    m.add_class::<TrackStuck>()?;
    m.add_class::<WebSocketClosed>()?;

    Ok(())
}
