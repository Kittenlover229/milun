use milun::Renderer;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Milun Quickstart Example")
        .build(&event_loop)
        .unwrap();

    let texture_16x16 = image::load_from_memory(include_bytes!("16x16.png"))
        .unwrap()
        .to_rgba8();
    let texture_8x8 = image::load_from_memory(include_bytes!("8x8.png"))
        .unwrap()
        .to_rgba8();

    let mut renderer = Renderer::from(window);
    renderer
        .atlas()
        .add_rgba8(texture_16x16)
        .add_rgba8(texture_8x8.clone())
        .add_rgba8(texture_8x8.clone())
        .finalize();

    event_loop.run(move |event, _, control_flow| {
        let window = renderer.window();

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !renderer.input(event) {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            renderer.resize(**new_inner_size);
                        }
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
                        _ => {}
                    }
                }
            }

            Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
                match renderer.render() {
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
