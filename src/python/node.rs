use crate::model::events::Events;

use std::sync::Arc;

use pyo3::prelude::*;

#[pymodule]
pub fn node(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Node>()?;
    m.add_class::<crate::node::NodeBuilder>()?;

    Ok(())
}

#[pyclass]
#[derive(Clone)]
pub(crate) struct Node {
    pub(crate) inner: Arc<crate::node::Node>,
}

#[pymethods]
impl Node {
    #[getter]
    fn http(&self) -> super::http::Http {
        super::http::Http {
            inner: self.inner.http.clone(),
        }
    }
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
    #[pyo3(signature = (hostname, is_ssl, password, user_id, session_id=None, events=None))]
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
            let current_loop = pyo3_async_runtimes::get_running_loop(py)?;
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
