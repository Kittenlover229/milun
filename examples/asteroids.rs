use mint::Vector2;
use tangerine::{Renderer, StandaloneDrawCallback, StandaloneRenderer};

const BACKGROUND: &str = "background";
const FOREGROUND: &str = "foreground";

struct Asteroid {
    pub position: Vector2<i32>,
    pub size: u32,
}

pub fn update_factory() -> impl StandaloneDrawCallback {
    move |renderer: &mut Renderer, input| {
        let mut frame = renderer.begin_frame();

        Ok(frame)
    }
}

fn main() {
    let mut renderer = StandaloneRenderer::new("Tangerine Asteroids Example");

    let mut asteroids: Vec<Asteroid> = vec![];

    renderer.set_layer(BACKGROUND, -1);
    renderer.set_layer(FOREGROUND, 1);

    renderer.run(update_factory()).unwrap();
}
