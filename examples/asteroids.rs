use std::{convert::Infallible, f32::consts::PI};

use mint::Vector2;
use tangerine::{
    Camera, FrameBuilder, SpriteInstance, SpriteTransform, StandaloneInputState, StandaloneRenderer,
};
use winit::event::{ScanCode, VirtualKeyCode};

const BACKGROUND: &str = "background";
const FOREGROUND: &str = "foreground";

pub const ROCK_SPINOR: f32 = 18.;

struct Asteroid {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub size: f32,

    pub rotor: f32,
}

struct Player {
    pub drag: f32,
    pub wish: Vector2<i8>,
    pub position: Vector2<f32>,
}

pub fn rect_wrap(mut v: Vector2<f32>, height: f32, width: f32, padding: f32) -> Vector2<f32> {
    if v.y - padding > height {
        v.y -= 2. * height + padding;
    }

    if v.y + padding < -height {
        v.y += 2. * height + padding;
    }

    if v.x - padding > width {
        v.x -= 2. * width + padding;
    }

    if v.x + padding < -width {
        v.x += 2. * width + padding;
    }

    v
}

fn handle_input(
    player: &mut Player,
    pressed: impl Iterator<Item = (ScanCode, VirtualKeyCode)>,
    released: impl Iterator<Item = (ScanCode, VirtualKeyCode)>,
) {
    for (_, keycode) in pressed {
        match keycode {
            VirtualKeyCode::W => {
                player.wish.y += 1;
            }
            VirtualKeyCode::S => {
                player.wish.y -= 1;
            }
            VirtualKeyCode::D => {
                player.wish.x += 1;
            }
            VirtualKeyCode::A => {
                player.wish.x -= 1;
            }
            _ => {}
        }
    }

    for (_, keycode) in released {
        match keycode {
            VirtualKeyCode::W => {
                player.wish.y -= 1;
            }
            VirtualKeyCode::S => {
                player.wish.y += 1;
            }
            VirtualKeyCode::D => {
                player.wish.x -= 1;
            }
            VirtualKeyCode::A => {
                player.wish.x += 1;
            }
            _ => {}
        }
    }
}

fn main() {
    let mut renderer = StandaloneRenderer::new("Tangerine Asteroids");

    let asteroid_texture = image::load_from_memory(include_bytes!("./assets/16x16.png")).unwrap();
    let spaceship_texture = image::load_from_memory(include_bytes!("./assets/8x8.png")).unwrap();
    let cursor_texture = image::load_from_memory(include_bytes!("./assets/8x8.png")).unwrap();

    let [asteroid_sprite, spaceship_sprite, cursor_sprite] = renderer
        .atlas()
        .add_sprite(asteroid_texture)
        .add_sprite(spaceship_texture)
        .add_sprite(cursor_texture)
        .finalize_and_repack();

    renderer.set_layer(BACKGROUND, -1);
    renderer.set_layer(FOREGROUND, 1);

    renderer.mutate_camera(|camera| camera.size = 8.);

    let mut asteroids: Vec<Asteroid> = vec![
        Asteroid {
            position: [0.; 2].into(),
            velocity: [0., 1.].into(),
            size: 1.,
            rotor: 90.,
        },
        Asteroid {
            position: [0.; 2].into(),
            velocity: [-0.707, 0.707].into(),
            size: 2.,
            rotor: 36.1,
        },
    ];

    let mut player = Player {
        drag: 0.7,
        position: [0.; 2].into(),
        wish: [0; 2].into(),
    };

    renderer
        .run::<Infallible>(
            move |frame: &mut FrameBuilder, input: &StandaloneInputState| {
                let Camera {
                    aspect_ratio, size, ..
                } = *frame.renderer().camera();

                let dt = input.delta_time_secs;

                handle_input(
                    &mut player,
                    input.pressed_keys.to_owned().into_iter(),
                    input.released_keys.to_owned().into_iter(),
                );

                let mut wish_x = player.wish.x as f32;
                let mut wish_y = player.wish.y as f32;
                let wish_len = (wish_x * wish_x + wish_y * wish_y).sqrt();
                if wish_len != 0. {
                    wish_x /= wish_len;
                    wish_y /= wish_len;
                    player.position.x += wish_x * dt;
                    player.position.y += wish_y * dt;
                }

                for asteroid in asteroids.iter_mut() {
                    asteroid.position.x += asteroid.velocity.x * dt;
                    asteroid.position.y += asteroid.velocity.y * dt;

                    asteroid.position =
                        rect_wrap(asteroid.position, size, size * aspect_ratio, asteroid.size);

                    asteroid.rotor += dt * size;

                    frame.draw_sprite(
                        asteroid_sprite,
                        FOREGROUND,
                        SpriteInstance {
                            position: [asteroid.position.x, asteroid.position.y, 0.].into(),
                            transform: SpriteTransform {
                                scale: [asteroid.size; 2].into(),
                                rotation_deg: ROCK_SPINOR * asteroid.rotor / 180. * PI,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    );
                }

                let cursor_pos: Vector2<f32> = frame.renderer().window_to_world(input.cursor_pos);
                frame.draw_sprite(
                    cursor_sprite,
                    FOREGROUND,
                    SpriteInstance {
                        position: [cursor_pos.x, cursor_pos.y, 0.].into(),
                        opacity: 0.333,
                        transform: SpriteTransform {
                            scale: [0.5; 2].into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                );

                player.position = rect_wrap(player.position, size, size * aspect_ratio, 1.);
                let angle =
                    -(player.position.x - cursor_pos.x).atan2(player.position.y - cursor_pos.y);

                frame.draw_sprite(
                    spaceship_sprite,
                    FOREGROUND,
                    SpriteInstance {
                        position: [player.position.x, player.position.y, 0.].into(),
                        transform: SpriteTransform {
                            rotation_deg: 180. * angle / PI,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                );

                Ok(())
            },
        )
        .unwrap();
}
