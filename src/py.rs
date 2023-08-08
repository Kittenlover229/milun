use std::cell::OnceCell;

use cint::EncodedSrgb;
use image::DynamicImage;
use mint::Vector2;
use pyo3::{buffer::PyBuffer, prelude::*, types::PyTuple};

use crate::{StandaloneInputState, SpriteIndex, SpriteTransform, StandaloneRenderer};

/// Input transferred to the Python's world.
#[pyclass(name = "Input", frozen, unsendable)]
pub struct PythonInput {
    gathered_input: StandaloneInputState,
    cursor_pos_world_space: Vector2<f32>,
}

#[pymethods]
impl PythonInput {
    #[getter]
    fn get_cursor_window_pos(&self, py: Python<'_>) -> PyObject {
        PyTuple::new(
            py,
            [
                self.gathered_input.cursor_pos.x,
                self.gathered_input.cursor_pos.y,
            ],
        )
        .to_object(py)
    }

    #[getter]
    fn get_cursor_world_pos(&self, py: Python<'_>) -> PyObject {
        PyTuple::new(
            py,
            [self.cursor_pos_world_space.x, self.cursor_pos_world_space.y],
        )
        .to_object(py)
    }
}

/// Adapter of the [`StandaloneRenderer`] to interact with Python's world.
#[pyclass(name = "Renderer", unsendable)]
pub struct PythonRenderer {
    new_background_color: OnceCell<Option<EncodedSrgb>>,
    new_title: OnceCell<String>,

    sprites_to_add: Vec<DynamicImage>,
    to_draw_list: Vec<(SpriteIndex, Vector2<f32>)>,

    last_sprite_index: SpriteIndex,
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
            last_sprite_index: 0,
        }
    }

    /// Hijack the main loop with the callback to call when a redraw is requested.
    pub fn run<'py>(
        slf: PyRefMut<'py, Self>,
        py: Python<'py>,
        redraw_callback: PyObject,
    ) -> PyResult<()> {
        StandaloneRenderer::new("Hello, Python!").run({
            let gil_pool = unsafe { py.new_pool() };
            let self_py = slf.into_py(py);

            move |renderer, gathered_input| {
                let python = gil_pool.python();
                let self_py: Py<Self> = self_py.extract::<Py<Self>>(python)?;
                let mut slf = self_py.borrow_mut(python);

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
                        slf.into_py(python),
                        PythonInput {
                            gathered_input,
                            cursor_pos_world_space,
                        },
                    ),
                )?;

                let mut self_borrow = self_py.borrow_mut(python);
                if !self_borrow.sprites_to_add.is_empty() {
                    let _ = std::mem::take(&mut self_borrow.sprites_to_add)
                        .into_iter()
                        .fold(renderer.atlas(), |r, i| r.add_sprite_dynamically(i).0)
                        .finalize_and_repack();
                }

                let mut frame_builder = renderer.begin_frame();

                let draw_list = &mut self_borrow.to_draw_list;
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

    /// Background color used for clearing the screen.
    /// Either Some(color) or None, when None is used
    /// the screen is not cleared.
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

    /// Queue a title update for the next frame
    fn set_title(&mut self, title: &str) {
        self.new_title = OnceCell::from(title.to_string())
    }

    /// Add a new sprite to the list of sprites for further drawing
    fn add_sprite(&mut self, py: Python<'_>, buffer: PyBuffer<u8>) -> SpriteIndex {
        let buffer = buffer
            .as_slice(py)
            .unwrap()
            .iter()
            .map(|x| x.get())
            .collect::<Vec<_>>();
        let img = image::load_from_memory(&buffer).unwrap();
        let new_idx = self.last_sprite_index;
        self.last_sprite_index += 1;
        self.sprites_to_add.push(img);
        new_idx
    }

    /// Add the sprite to the drawing queue
    fn draw(&mut self, py: Python<'_>, index: SpriteIndex, at: PyObject) -> PyResult<()> {
        self.to_draw_list
            .push((index, Vector2::from(at.extract::<[f32; 2]>(py)?)));
        Ok(())
    }
}

#[pymodule]
fn wffle(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PythonRenderer>()?;
    module.add_class::<PythonInput>()?;
    Ok(())
}
