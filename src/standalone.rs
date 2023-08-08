use std::{
    cell::OnceCell,
    error::Error,
    ops::{Deref, DerefMut},
};

use mint::Vector2;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use crate::{FrameBuilder, Renderer};

pub struct StandaloneRenderer {
    pub renderer: Renderer,
    pub event_loop: EventLoop<()>,
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct GatheredInput {
    pub cursor_pos: Vector2<u32>,
}

impl Default for GatheredInput {
    fn default() -> Self {
        Self {
            cursor_pos: [0; 2].into(),
        }
    }
}

impl StandaloneRenderer {
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

impl StandaloneRenderer {
    pub fn run<E: Error>(
        self,
        mut draw_callback: impl FnMut(&mut Renderer, GatheredInput) -> Result<FrameBuilder<'_>, E>
            + 'static,
    ) -> Result<(), E> {
        let StandaloneRenderer {
            mut event_loop,
            mut renderer,
            ..
        } = self;

        let mut gathered_input = GatheredInput {
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
