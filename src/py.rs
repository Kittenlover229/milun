use pyo3::prelude::*;

use crate::StandaloneRenderer;

#[pyclass(name = "Renderer", unsendable)]
pub struct PythonRenderer {}

#[pymethods]
impl PythonRenderer {
    #[new]
    pub fn new() -> PyResult<Self> {
        Ok(Self {})
    }

    pub fn run(&self, py: Python<'_>, redraw_callback: PyObject) -> PyResult<()> {
        let gil_pool = unsafe { py.new_pool() };
        StandaloneRenderer::new("Hello, Python!").run(move |renderer, _input| {
            redraw_callback.call0(gil_pool.python())?;
            Ok(renderer.begin_frame())
        })
    }
}

#[pymodule]
fn wffle(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PythonRenderer>()?;
    Ok(())
}
