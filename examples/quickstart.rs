use wffle::{SpriteTransform, StandaloneRenderer};

fn main() {
    let mut renderer = StandaloneRenderer::new("Wffle Quickstart Example");

    let texture_16x16 = image::load_from_memory(include_bytes!("16x16.png")).unwrap();
    let texture_8x8 = image::load_from_memory(include_bytes!("8x8.png")).unwrap();
    let texture_8x16 = image::load_from_memory(include_bytes!("8x16.png")).unwrap();
    let [i16x16, i8x16, i8x8] = renderer
        .atlas()
        .add_sprite(texture_16x16)
        .add_sprite(texture_8x16)
        .add_sprite(texture_8x8)
        .finalize_and_repack();

    renderer.run(move |renderer, input| {
        let cursor_pos = renderer.window_to_world(input.cursor_pos);
        let frame = renderer.begin_frame();

        frame
            .draw_sprite_indexed(
                i8x8,
                [0., 0.],
                SpriteTransform::default(),
                [0x00, 0xFF, 0xFF],
                1.0,
            )
            .draw_sprite_indexed(
                i8x8,
                cursor_pos,
                SpriteTransform::scaled([1., 2.]),
                [0xFF, 0xFF, 0xFF],
                1.0,
            )
            .draw_sprite_indexed(
                i16x16,
                [0., 1.],
                SpriteTransform::default(),
                [0xFF, 0xFF, 0x00],
                1.0,
            )
            .draw_sprite_indexed(
                i8x16,
                [0., 2.],
                SpriteTransform::default(),
                [0xFF, 0x00, 0xFF],
                0.5,
            )
            .draw_egui(|ctx| {
                egui::Window::new("Demo Egui Window").show(ctx, |ui| {
                    ui.label("Hello, world!");
                });
            })
    });
}
