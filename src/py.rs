use std::cell::OnceCell;

use cint::EncodedSrgb;
use pyo3::prelude::*;

use crate::StandaloneRenderer;

#[pyclass(name = "Renderer", unsendable)]
pub struct PythonRenderer {
    new_background_color: OnceCell<Option<EncodedSrgb>>,
}

#[pymethods]
impl PythonRenderer {
    #[new]
    pub fn new(_py: Python<'_>) -> Self {
        Self {
            new_background_color: OnceCell::new(),
        }
    }

    pub fn run<'py>(
        slf: PyRefMut<'py, Self>,
        py: Python<'py>,
        redraw_callback: PyObject,
    ) -> PyResult<()> {
        StandaloneRenderer::new("Hello, Python!").run({
            let gil_pool = unsafe { py.new_pool() };
            let slf = slf.into_py(py);

            move |renderer, _input| {
                let python = gil_pool.python();
                let ref_slf = slf.clone();
                redraw_callback.call1(python, (ref_slf.as_ref(python),))?;
                let x: Py<Self> = slf.extract(python)?;
                let mut z = x.borrow_mut(python);
                if let Some(color) = z.new_background_color.take() {
                    renderer.clear_color = color;
                }

                Ok(renderer.begin_frame())
            }
        })
    }

    pub fn set_background_color<'py>(
        mut slf: PyRefMut<'py, Self>,
        py: Python<'py>,
        colors: PyObject,
    ) -> PyResult<()> {
        let py_colors: &PyAny = colors.as_ref(py);
        slf.new_background_color = OnceCell::from(if py_colors.is_none() {
            None
        } else {
            let colors = match py_colors.extract::<[u8; 3]>() {
                Ok(arr) => arr,
                Err(_) => py_colors.extract::<[f32; 3]>()?.map(|x| (x * 255.) as u8),
            };

            Some(EncodedSrgb::from(colors))
        });

        Ok(())
    }
}

#[pymodule]
fn wffle(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PythonRenderer>()?;
    Ok(())
}
