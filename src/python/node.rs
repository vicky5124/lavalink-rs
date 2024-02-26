use crate::model::events::Events;

use pyo3::prelude::*;

#[pymodule]
pub fn node(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<crate::node::NodeBuilder>()?;

    Ok(())
}

#[apply(super::with_getter_setter)]
#[pymethods]
impl crate::node::NodeBuilder {
    getter_setter!(
        (hostname, String),
        (is_ssl, bool),
        (password, String),
        (user_id, crate::model::UserId),
        (session_id, Option<String>),
    );

    #[new]
    fn new(
        py: Python<'_>,
        hostname: String,
        is_ssl: bool,
        password: String,
        user_id: super::model::PyUserId,
        session_id: Option<String>,
        events: Option<PyObject>,
    ) -> PyResult<Self> {
        let events = if let Some(events) = events {
            let current_loop = pyo3_asyncio::get_running_loop(py)?;
            let loop_ref = PyObject::from(current_loop);

            let event_handler = crate::python::event::EventHandler {
                inner: events,
                current_loop: loop_ref,
            };

            Events {
                event_handler: Some(event_handler),
                ..Default::default()
            }
        } else {
            Events::default()
        };

        Ok(crate::node::NodeBuilder {
            hostname,
            is_ssl,
            events,
            password,
            user_id: user_id.into(),
            session_id,
        })
    }
}
