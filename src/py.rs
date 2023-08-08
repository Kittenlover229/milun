use std::cell::OnceCell;

use cint::EncodedSrgb;
use mint::Vector2;
use pyo3::{prelude::*, types::PyTuple};

use crate::{GatheredInput, StandaloneRenderer};

#[pyclass(name = "Input", frozen, unsendable)]
pub struct Input {
    gathered_input: GatheredInput,
    cursor_pos_world_space: Vector2<f32>,
}

#[pymethods]
impl Input {
    #[getter]
    fn get_cursor_window_pos(&self, py: Python<'_>) -> PyObject {
        PyTuple::new(
            py,
            [
                self.gathered_input.cursor_pos.x,
                self.gathered_input.cursor_pos.y,
            ]
            .into_iter(),
        )
        .to_object(py)
    }

    #[getter]
    fn get_cursor_world_pos(&self, py: Python<'_>) -> PyObject {
        PyTuple::new(
            py,
            [self.cursor_pos_world_space.x, self.cursor_pos_world_space.y].into_iter(),
        )
        .to_object(py)
    }
}

#[pyclass(name = "Renderer", unsendable)]
pub struct PythonRenderer {
    new_background_color: OnceCell<Option<EncodedSrgb>>,
    new_title: OnceCell<String>,
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

            move |renderer, gathered_input| {
                let python = gil_pool.python();
                let slf: Py<Self> = slf.extract::<Py<Self>>(python)?;
                let mut slf = slf.borrow_mut(python);

                if let Some(color) = slf.new_background_color.take() {
                    renderer.clear_color = color;
                }

                if let Some(title) = slf.new_title.take() {
                    renderer.window.set_title(&title);
                }

                let cursor_pos_world_space = renderer.window_to_world(gathered_input.cursor_pos);

                redraw_callback.call1(
                    python,
                    (
                        slf,
                        Input {
                            gathered_input,
                            cursor_pos_world_space,
                        },
                    ),
                )?;

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
