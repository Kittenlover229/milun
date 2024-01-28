use tangerine::{SpriteInstance, StandaloneRenderer};

const BACKGROUND: &str = "Some(BACKGROUND)";
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

    renderer.run_infallible(move |frame, input| {
        let cursor_pos = frame.renderer().window_to_world(input.cursor_pos);

        frame
            .draw_sprite(
                i8x8,
                Some(BACKGROUND),
                SpriteInstance {
                    color: [0x00, 0xFF, 0xFF].into(),
                    ..Default::default()
                },
            )
            .draw_sprite(
                i8x8,
                Some(FOREGROUND),
                SpriteInstance {
                    position: [cursor_pos.x, cursor_pos.y, 0.].into(),
                    color: [0xFF, 0xFF, 0xFF].into(),
                    ..Default::default()
                },
            )
            .draw_sprite(
                i16x16,
                Some(BACKGROUND),
                SpriteInstance {
                    position: [0., 1., 0.].into(),
                    color: [0xFF, 0xFF, 0x00].into(),
                    ..Default::default()
                },
            )
            .draw_sprite(
                i16x16,
                Some(BACKGROUND),
                SpriteInstance {
                    position: [1., 0., 0.].into(),
                    color: [0xFF, 0xFF, 0x00].into(),
                    ..Default::default()
                },
            )
            .draw_sprite(
                i8x16,
                Some(BACKGROUND),
                SpriteInstance {
                    position: [0., 2., 0.].into(),
                    opacity: 0.5,
                    ..Default::default()
                },
            )
            .done();
    });
}
