use wffle::Renderer;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WFFLE Quickstart Example")
        .build(&event_loop)
        .unwrap();

    let texture_16x16 = image::load_from_memory(include_bytes!("16x16.png")).unwrap();
    let texture_8x8 = image::load_from_memory(include_bytes!("8x8.png")).unwrap();
    let texture_8x16 = image::load_from_memory(include_bytes!("8x16.png")).unwrap();

    let mut renderer = Renderer::from(window);
    let [i16x16, i8x16, i8x8] = renderer
        .atlas()
        .add_sprite(texture_16x16)
        .add_sprite(texture_8x16)
        .add_sprite(texture_8x8)
        .finalize_and_repack();
    let mut cursor_pos = [0u32; 2];

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
                            cursor_pos = [position.x as u32, position.y as u32].into();
                        }
                        _ => {}
                    }
                }
            }

            Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
                let cursor_pos = renderer.window_to_world(cursor_pos);
                let result = renderer
                    .begin_frame()
                    .draw_sprite_indexed(i8x8, [0., 0.], [0x00, 0xFF, 0xFF], 1.0)
                    .draw_sprite_indexed(i8x8, cursor_pos, [0xFF, 0xFF, 0xFF], 1.0)
                    .draw_sprite_indexed(i16x16, [0., 1.], [0xFF, 0xFF, 0x00], 1.0)
                    .draw_sprite_indexed(i8x16, [0., 2.], [0xFF, 0x00, 0xFF], 0.5)
                    .draw_egui(|ctx| {
                        egui::Window::new("Demo Egui Window").show(ctx, |ui| {
                            ui.label("Hello, world!");
                        });
                    })
                    .end_frame();

                match result {
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
