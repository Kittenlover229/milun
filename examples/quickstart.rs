use std::convert::Infallible;

use wffle::{FrameBuilder, SpriteInstance, StandaloneRenderer};

fn main() {
    let mut renderer = StandaloneRenderer::new("Wffle Quickstart Example");
    renderer.set_layer("background", 0);
    renderer.set_layer("foreground", 5);

    let texture_16x16 = image::load_from_memory(include_bytes!("16x16.png")).unwrap();
    let texture_8x16 = image::load_from_memory(include_bytes!("8x16.png")).unwrap();
    let texture_8x8 = image::load_from_memory(include_bytes!("8x8.png")).unwrap();

    let [i16x16, i8x16, i8x8] = renderer
        .atlas()
        .add_sprite(texture_16x16)
        .add_sprite(texture_8x16)
        .add_sprite(texture_8x8)
        .finalize_and_repack();

    renderer
        .run(move |renderer, input| {
            let cursor_pos = renderer.window_to_world(input.cursor_pos);
            let frame = renderer.begin_frame();

            Ok::<FrameBuilder<'_>, Infallible>(
                frame
                    .draw_sprite_indexed(
                        i8x8,
                        "background",
                        SpriteInstance {
                            color: [0x00, 0xFF, 0xFF].into(),
                            ..Default::default()
                        },
                    )
                    .draw_sprite_indexed(
                        i8x8,
                        "foreground",
                        SpriteInstance {
                            position: cursor_pos,
                            color: [0xFF, 0xFF, 0xFF].into(),
                            ..Default::default()
                        },
                    )
                    .draw_sprite_indexed(
                        i16x16,
                        "background",
                        SpriteInstance {
                            position: [0., 1.].into(),
                            color: [0xFF, 0xFF, 0x00].into(),
                            ..Default::default()
                        },
                    )
                    .draw_sprite_indexed(
                        i8x16,
                        "background",
                        SpriteInstance {
                            position: [0., 2.].into(),
                            opacity: 0.5,
                            ..Default::default()
                        },
                    )
                    .draw_egui(|ctx| {
                        egui::Window::new("Demo Egui Window").show(ctx, |ui| {
                            ui.label("Hello, world!");
                        });
                    }),
            )
        })
        .unwrap();
}
