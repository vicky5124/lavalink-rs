use crate::model::events::Events;

use pyo3::prelude::*;

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
        hostname: String,
        is_ssl: bool,
        password: String,
        user_id: super::model::PyUserId,
        session_id: Option<String>,
    ) -> Self {
        crate::node::NodeBuilder {
            hostname,
            is_ssl,
            events: Events::default(),
            password,
            user_id: user_id.into(),
            session_id,
        }
    }
}
