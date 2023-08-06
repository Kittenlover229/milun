use std::ops::{Deref, DerefMut};

use mint::Vector2;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
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
    pub fn run(
        self,
        mut draw_callback: impl FnMut(&mut Renderer, GatheredInput) -> FrameBuilder<'_> + 'static,
    ) -> ! {
        let StandaloneRenderer {
            event_loop,
            mut renderer,
            ..
        } = self;

        let mut gathered_input = GatheredInput {
            cursor_pos: [0; 2].into(),
        };

        event_loop.run(move |event, _, control_flow| {
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
                    match draw_callback(&mut renderer, gathered_input).end_frame() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size()),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                }

                Event::MainEventsCleared => {
                    renderer.window().request_redraw();
                }

                _ => {}
            }
        });
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
