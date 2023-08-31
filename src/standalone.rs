use std::{
    cell::OnceCell,
    error::Error,
    ops::{Deref, DerefMut}, convert::Infallible,
};

use mint::Vector2;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use crate::{FrameBuilder, Renderer};

/// Standalone renderer that instead of taking ownership of an existing window creates its own.
pub struct StandaloneRenderer {
    /// Main renderer in control of the window.
    pub renderer: Renderer,
    /// Event loop in which the `.run(..)` function is ran.
    pub event_loop: EventLoop<()>,
}

/// All the input gathered by the [`StandaloneRenderer`] since last frame.
#[derive(Debug, Clone, Copy, Hash)]
#[repr(C)]
pub struct StandaloneInputState {
    /// Last recorded position of the cursor in window space.
    pub cursor_pos: Vector2<u32>,
}

impl Default for StandaloneInputState {
    fn default() -> Self {
        Self {
            cursor_pos: [0; 2].into(),
        }
    }
}

impl StandaloneRenderer {
    /// Create a new standalone renderer with the provided window title.
    pub fn new(window_title: impl Into<String>) -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(window_title)
            .build(&event_loop)
            .unwrap();

        Self {
            renderer: Renderer::from(window),
            event_loop,
        }
    }
}

pub trait StandaloneDrawCallback<E: Error = Infallible> =
    FnMut(&mut Renderer, StandaloneInputState) -> Result<FrameBuilder<'_>, E>;

impl StandaloneRenderer {
    /// Run the event loop until an error is encountered, close request is received or `Esc` is pressed.
    pub fn run<E: Error>(self, mut draw_callback: impl StandaloneDrawCallback<E> + 'static) -> Result<(), E> {
        let StandaloneRenderer {
            mut event_loop,
            mut renderer,
            ..
        } = self;

        let mut gathered_input = StandaloneInputState {
            cursor_pos: [0; 2].into(),
        };

        let mut error_return: OnceCell<E> = Default::default();

        event_loop.run_return({
            let error_return = &mut error_return;
            move |event, _, control_flow| {
                let window = renderer.window();
                match event {
                    Event::WindowEvent {
                        ref event,
                        window_id,
                    } if window_id == window.id() => {
                        if !renderer.input(event) {
                            match event {
                                WindowEvent::CloseRequested
                                | WindowEvent::KeyboardInput {
                                    input:
                                        KeyboardInput {
                                            state: ElementState::Pressed,
                                            virtual_keycode: Some(VirtualKeyCode::Escape),
                                            ..
                                        },
                                    ..
                                } => *control_flow = ControlFlow::Exit,
                                WindowEvent::CursorMoved { position, .. } => {
                                    gathered_input.cursor_pos =
                                        [position.x as u32, position.y as u32].into();
                                }
                                _ => {}
                            }
                        }
                    }

                    Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
                        let draw_result = match draw_callback(&mut renderer, gathered_input) {
                            Ok(res) => res,
                            Err(err) => {
                                *error_return = err.into();
                                *control_flow = ControlFlow::Exit;
                                return;
                            }
                        };

                        match draw_result.end_frame() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size()),
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                *control_flow = ControlFlow::Exit
                            }
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }

                    Event::MainEventsCleared => {
                        renderer.window().request_redraw();
                    }

                    _ => {}
                }
            }
        });

        match error_return.into_inner() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

impl Deref for StandaloneRenderer {
    type Target = Renderer;

    fn deref(&self) -> &Self::Target {
        &self.renderer
    }
}

impl DerefMut for StandaloneRenderer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.renderer
    }
}
