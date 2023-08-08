use std::{borrow::BorrowMut, cell::OnceCell};

use cint::EncodedSrgb;
use image::DynamicImage;
use mint::Vector2;
use pyo3::{buffer::PyBuffer, prelude::*, types::PyTuple};

use crate::{GatheredInput, SpriteIndex, SpriteTransform, StandaloneRenderer};

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

    sprites_to_add: Vec<DynamicImage>,
    to_draw_list: Vec<(SpriteIndex, Vector2<f32>)>,
}

#[pymethods]
impl PythonRenderer {
    #[new]
    pub fn new(_py: Python<'_>) -> Self {
        Self {
            new_background_color: OnceCell::new(),
            new_title: OnceCell::new(),
            sprites_to_add: vec![],
            to_draw_list: vec![],
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

                if !slf.sprites_to_add.is_empty() {
                    let mut indices = vec![];
                    let _ = std::mem::replace(&mut slf.sprites_to_add, vec![])
                        .into_iter()
                        .fold(renderer.atlas(), |r, i| {
                            r.add_sprite_dynamically(i, &mut indices)
                        })
                        .finalize_and_repack();
                }

                let cursor_pos_world_space = renderer.window_to_world(gathered_input.cursor_pos);

                let x = slf.into_py(python);
                redraw_callback.call1(
                    python,
                    (
                        x.clone(),
                        Input {
                            gathered_input,
                            cursor_pos_world_space,
                        },
                    ),
                )?;

                let mut frame_builder = renderer.begin_frame();

                let slf = x.extract::<Py<Self>>(python)?;
                let draw_list = &mut slf.borrow_mut(python).to_draw_list;
                for (draw_idx, pos) in draw_list.iter() {
                    frame_builder = frame_builder.draw_sprite_indexed(
                        *draw_idx,
                        *pos,
                        SpriteTransform::default(),
                        [0xFF; 3],
                        1.,
                    )
                }
                draw_list.clear();

                Ok(frame_builder)
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

    fn add_sprite<'py>(&mut self, py: Python<'py>, buffer: PyBuffer<u8>) {
        let buffer = buffer
            .as_slice(py)
            .unwrap()
            .into_iter()
            .map(|x| x.get())
            .collect::<Vec<_>>();
        let img = image::load_from_memory(&buffer).unwrap();
        self.sprites_to_add.push(img);
    }

    fn draw(&mut self, py: Python<'_>, index: SpriteIndex, at: PyObject) -> PyResult<()> {
        self.to_draw_list
            .push((index, Vector2::from(at.extract::<[f32; 2]>(py)?)));
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
