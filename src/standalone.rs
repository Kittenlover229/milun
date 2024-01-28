use std::{
    cell::OnceCell,
    convert::Infallible,
    error::Error,
    ops::{Deref, DerefMut},
};

use hashbrown::HashSet;
use mint::Vector2;
use smallvec::{smallvec, SmallVec};
use winit::{
    event::{ElementState, Event, KeyboardInput, ScanCode, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use crate::{FrameBuilder, Renderer};

/// Standalone renderer that instead of taking ownership of an existing window creates its own.
pub struct StandaloneRenderer {
    /// Main renderer in control of the window.
    renderer: Renderer,
    /// Event loop in which the `.run(..)` function is ran.
    event_loop: EventLoop<()>,
    /// Map of the active states of all currently pressed keys
    keys_pressed: HashSet<VirtualKeyCode>,
}

/// All the input gathered by the [`StandaloneRenderer`] since last frame.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct StandaloneInputState {
    /// Last recorded position of the cursor in window space.
    pub cursor_pos: Vector2<u32>,
    /// The amount of time since the last render (in seconds).
    pub delta_time_secs: f32,
    /// All the keys that were pressed since last frame.
    pub pressed_keys: SmallVec<[(ScanCode, VirtualKeyCode); 4]>,
    /// All the keys that were released since last frame.
    pub released_keys: SmallVec<[(ScanCode, VirtualKeyCode); 4]>,
}

impl Default for StandaloneInputState {
    fn default() -> Self {
        Self {
            cursor_pos: [0; 2].into(),
            delta_time_secs: 0.,
            pressed_keys: smallvec![],
            released_keys: smallvec![],
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
            keys_pressed: Default::default(),
        }
    }
}

pub trait StandaloneDrawCallback<E: Error = Infallible> =
    for<'a, 'b> FnMut(&'b mut FrameBuilder<'a>, &StandaloneInputState) -> Result<(), E>;

pub trait InfallibleDrawCallback =
    for<'a, 'b> FnMut(&'b mut FrameBuilder<'a>, &StandaloneInputState);

impl StandaloneRenderer {
    pub fn run_infallible(self, mut draw_callback: impl InfallibleDrawCallback + 'static) {
        self.run(move |a, b| Ok::<(), Infallible>(draw_callback(a, b)))
            .unwrap();
    }

    /// Run the event loop until an error is encountered, close request is received when `Esc` is pressed.
    pub fn run<E: Error>(
        self,
        mut draw_callback: impl StandaloneDrawCallback<E> + 'static,
    ) -> Result<(), E> {
        let StandaloneRenderer {
            mut event_loop,
            mut renderer,
            mut keys_pressed,
            ..
        } = self;

        let mut gathered_input = StandaloneInputState {
            cursor_pos: [0; 2].into(),
            delta_time_secs: 0.,
            pressed_keys: smallvec![],
            released_keys: smallvec![],
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
                                WindowEvent::KeyboardInput {
                                    input:
                                        KeyboardInput {
                                            scancode,
                                            state,
                                            virtual_keycode: Some(keycode),
                                            ..
                                        },
                                    ..
                                } => match state {
                                    ElementState::Pressed if !keys_pressed.contains(keycode) => {
                                        gathered_input.pressed_keys.push((*scancode, *keycode));
                                        keys_pressed.insert(*keycode);
                                    }
                                    ElementState::Released => {
                                        gathered_input.released_keys.push((*scancode, *keycode));
                                        keys_pressed.remove(keycode);
                                    }
                                    _ => {}
                                },
                                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                                WindowEvent::CursorMoved { position, .. } => {
                                    gathered_input.cursor_pos =
                                        [position.x as u32, position.y as u32].into();
                                }
                                _ => {}
                            }
                        }
                    }

                    Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
                        gathered_input.delta_time_secs = renderer
                            .delta_time
                            .to_owned()
                            .num_nanoseconds()
                            .unwrap_or_default()
                            as f32
                            * (10e-9);

                        let mut frame_builder = renderer.begin_frame();

                        match draw_callback(&mut frame_builder, &gathered_input) {
                            Err(err) => {
                                *error_return = err.into();
                                *control_flow = ControlFlow::Exit;
                                return;
                            }
                            _ => {}
                        };

                        gathered_input.pressed_keys.clear();
                        gathered_input.released_keys.clear();

                        match frame_builder.end_frame() {
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
