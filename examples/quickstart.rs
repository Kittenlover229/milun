use tangerine::StandaloneRenderer;

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

    renderer.run_infallible(move |frame, input| {
        let cursor_pos = frame.viewport().window_to_world(input.cursor_pos);

        frame
            .draw_sprite(i8x8)
            .layer(FOREGROUND)
            .pos([cursor_pos.x, cursor_pos.y, 0.])
            .color([255; 3])
            .done();
    });
}
