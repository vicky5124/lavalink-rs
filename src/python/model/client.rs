use crate::model::client::NodeDistributionStrategy;
use pyo3::prelude::*;

pub fn client(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    let client = PyModule::new(py, "client")?;

    client.add_class::<NodeDistributionStrategyPy>()?;

    m.add_submodule(client)?;

    Ok(())
}

#[pyclass(name = "NodeDistributionStrategy")]
#[derive(Clone)]
pub(crate) struct NodeDistributionStrategyPy {
    pub(crate) inner: NodeDistributionStrategy,
}

#[pymethods]
impl NodeDistributionStrategyPy {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: NodeDistributionStrategy::new(),
        }
    }

    #[staticmethod]
    pub fn sharded() -> Self {
        Self {
            inner: NodeDistributionStrategy::sharded(),
        }
    }

    #[staticmethod]
    pub fn round_robin() -> Self {
        Self {
            inner: NodeDistributionStrategy::round_robin(),
        }
    }

    #[staticmethod]
    pub fn main_fallback() -> Self {
        Self {
            inner: NodeDistributionStrategy::main_fallback(),
        }
    }

    #[staticmethod]
    pub fn lowest_load() -> Self {
        Self {
            inner: NodeDistributionStrategy::lowest_load(),
        }
    }

    #[staticmethod]
    pub fn highest_free_memory() -> Self {
        Self {
            inner: NodeDistributionStrategy::highest_free_memory(),
        }
    }

    #[staticmethod]
    pub fn custom(func: PyObject) -> Self {
        Self {
            inner: NodeDistributionStrategy::CustomPython(func),
        }
    }
}
