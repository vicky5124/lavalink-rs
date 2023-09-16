use crate::model::player::*;

use pyo3::prelude::*;

#[pymodule]
pub fn player(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Player>()?;
    m.add_class::<State>()?;
    m.add_class::<ConnectionInfo>()?;
    m.add_class::<Filters>()?;
    m.add_class::<ChannelMix>()?;
    m.add_class::<Distortion>()?;
    m.add_class::<Equalizer>()?;
    m.add_class::<Karaoke>()?;
    m.add_class::<LowPass>()?;
    m.add_class::<Rotation>()?;
    m.add_class::<Timescale>()?;
    m.add_class::<TremoloVibrato>()?;

    Ok(())
}

#[apply(crate::python::with_getter_setter)]
#[pymethods]
impl Filters {
    getter_setter!(
        (volume, Option<u16>),
        (equalizer, Option<Vec<Equalizer>>),
        (karaoke, Option<Karaoke>),
        (timescale, Option<Timescale>),
        (tremolo, Option<TremoloVibrato>),
        (vibrato, Option<TremoloVibrato>),
        (rotation, Option<Rotation>),
        (distortion, Option<Distortion>),
        (channel_mix, Option<ChannelMix>),
        (low_pass, Option<LowPass>),
    );
}
