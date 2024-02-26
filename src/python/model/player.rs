use crate::model::player::*;

use pyo3::prelude::*;

pub fn player(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    let player = PyModule::new(py, "player")?;

    player.add_class::<Player>()?;
    player.add_class::<State>()?;
    player.add_class::<ConnectionInfo>()?;
    player.add_class::<Filters>()?;
    player.add_class::<ChannelMix>()?;
    player.add_class::<Distortion>()?;
    player.add_class::<Equalizer>()?;
    player.add_class::<Karaoke>()?;
    player.add_class::<LowPass>()?;
    player.add_class::<Rotation>()?;
    player.add_class::<Timescale>()?;
    player.add_class::<TremoloVibrato>()?;

    m.add_submodule(player)?;

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
