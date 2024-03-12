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

#[pymethods]
impl ConnectionInfo {
    #[new]
    fn new_py(endpoint: String, token: String, session_id: String) -> ConnectionInfo {
        ConnectionInfo {
            endpoint,
            token,
            session_id,
        }
    }

    #[pyo3(name = "fix")]
    fn fix_py(&mut self) {
        self.fix()
    }
}

#[pymethods]
impl ChannelMix {
    #[new]
    fn new_py() -> ChannelMix {
        ChannelMix::default()
    }
}

#[pymethods]
impl Distortion {
    #[new]
    fn new_py() -> Distortion {
        Distortion::default()
    }
}

#[pymethods]
impl Equalizer {
    #[new]
    fn new_py() -> Equalizer {
        Equalizer::default()
    }
}

#[pymethods]
impl Karaoke {
    #[new]
    fn new_py() -> Karaoke {
        Karaoke::default()
    }
}

#[pymethods]
impl LowPass {
    #[new]
    fn new_py() -> LowPass {
        LowPass::default()
    }
}

#[pymethods]
impl Rotation {
    #[new]
    fn new_py() -> Rotation {
        Rotation::default()
    }
}

#[pymethods]
impl Timescale {
    #[new]
    fn new_py() -> Timescale {
        Timescale::default()
    }
}

#[pymethods]
impl TremoloVibrato {
    #[new]
    fn new_py() -> TremoloVibrato {
        TremoloVibrato::default()
    }
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

    #[new]
    fn new_py() -> Filters {
        Filters::default()
    }
}
