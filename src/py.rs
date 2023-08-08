use std::cell::OnceCell;

use cint::EncodedSrgb;
use pyo3::prelude::*;

use crate::{StandaloneRenderer, GatheredInput};

#[pyclass(name = "Renderer", unsendable)]
pub struct PythonRenderer {
    new_background_color: OnceCell<Option<EncodedSrgb>>,
    new_title: OnceCell<String>,
}

#[pyclass(name = "GatheredInput", frozen)]
pub struct Input {}

impl From<GatheredInput> for Input {
    fn from(value: GatheredInput) -> Self {
        Input {  }
    }
}

#[pymethods]
impl PythonRenderer {
    #[new]
    pub fn new(_py: Python<'_>) -> Self {
        Self {
            new_background_color: OnceCell::new(),
            new_title: OnceCell::new(),
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

            move |renderer, input| {
                let python = gil_pool.python();
                let slf: Py<Self> = slf.extract::<Py<Self>>(python)?;
                let mut slf = slf.borrow_mut(python);

                if let Some(color) = slf.new_background_color.take() {
                    renderer.clear_color = color;
                }

                if let Some(title) = slf.new_title.take() {
                    renderer.window.set_title(&title);
                }

                redraw_callback.call1(python, (slf, Input::from(input)))?;

                Ok(renderer.begin_frame())
            }
        })
    }

    fn set_background_color<'py>(
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

    fn set_title(&mut self, title: &str) {
        self.new_title = OnceCell::from(title.to_string())
    }
}

#[pymodule]
fn wffle(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PythonRenderer>()?;
    Ok(())
}
