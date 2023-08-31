use std::convert::Infallible;

use tangerine::{SpriteInstance, StandaloneRenderer};

const BACKGROUND: &str = "background";
const FOREGROUND: &str = "foreground";

fn main() {
    let mut renderer = StandaloneRenderer::new("Tangerine Quickstart Example");
    renderer.set_layer(BACKGROUND, 0);
    renderer.set_layer(FOREGROUND, 5);

    let texture_16x16 = image::load_from_memory(include_bytes!("./assets/16x16.png")).unwrap();
    let texture_8x16 = image::load_from_memory(include_bytes!("./assets/8x16.png")).unwrap();
    let texture_8x8 = image::load_from_memory(include_bytes!("./assets/8x8.png")).unwrap();

    let [i16x16, i8x16, i8x8] = renderer
        .atlas()
        .add_sprite(texture_16x16)
        .add_sprite(texture_8x16)
        .add_sprite(texture_8x8)
        .finalize_and_repack();

    renderer
        .run::<Infallible>(move |renderer, input| {
            let cursor_pos = renderer.window_to_world(input.cursor_pos);
            let frame = renderer.begin_frame();

            let frame = frame
                .draw_sprite(
                    i8x8,
                    BACKGROUND,
                    SpriteInstance {
                        color: [0x00, 0xFF, 0xFF].into(),
                        ..Default::default()
                    },
                )
                .draw_sprite(
                    i8x8,
                    FOREGROUND,
                    SpriteInstance {
                        position: [cursor_pos.x, cursor_pos.y, 0.].into(),
                        color: [0xFF, 0xFF, 0xFF].into(),
                        ..Default::default()
                    },
                )
                .draw_sprite(
                    i16x16,
                    BACKGROUND,
                    SpriteInstance {
                        position: [0., 1., 0.].into(),
                        color: [0xFF, 0xFF, 0x00].into(),
                        ..Default::default()
                    },
                )
                .draw_sprite(
                    i16x16,
                    BACKGROUND,
                    SpriteInstance {
                        position: [1., 0., 0.].into(),
                        color: [0xFF, 0xFF, 0x00].into(),
                        ..Default::default()
                    },
                )
                .draw_sprite(
                    i8x16,
                    BACKGROUND,
                    SpriteInstance {
                        position: [0., 2., 0.].into(),
                        opacity: 0.5,
                        ..Default::default()
                    },
                );

            Ok(frame)
        })
        .unwrap();
}
