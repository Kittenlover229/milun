use std::f32::consts::{PI, TAU};

use mint::Vector2;
use tangerine::{
    Camera, FrameBuilder, InfallibleDrawCallback, SpriteIndex, StandaloneInputState,
    StandaloneRenderer,
};
use winit::event::{ScanCode, VirtualKeyCode};

const BACKGROUND: &str = "background";
const FOREGROUND: &str = "foreground";
const PROJECTILE_SPEED: f32 = 2.;

pub const ROCK_SPINOR: f32 = 0.1 / TAU;

struct Asteroid {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub size: f32,
    pub rotor: f32,
}

struct Player {
    pub drag: f32,
    pub wish: Vector2<i8>,
    pub velocity: Vector2<f32>,
    pub position: Vector2<f32>,
}

struct Projectile {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
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

fn handle_input<'a, 'b>(
    player: &mut Player,
    pressed: impl Iterator<Item = &'a (ScanCode, VirtualKeyCode)>,
    released: impl Iterator<Item = &'b (ScanCode, VirtualKeyCode)>,
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

pub fn make_draw_callback(
    asteroid_sprite: SpriteIndex,
    spaceship_sprite: SpriteIndex,
    cursor_sprite: SpriteIndex,
    projectile_sprite: SpriteIndex,
) -> impl InfallibleDrawCallback {
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

    let mut projectiles: Vec<Projectile> = vec![];

    let mut player = Player {
        drag: 0.99,
        position: [0.; 2].into(),
        wish: [0; 2].into(),
        velocity: [0.; 2].into(),
    };

    move |frame: &mut FrameBuilder, input: &StandaloneInputState| {
        let Camera {
            aspect_ratio, size, ..
        } = *frame.renderer().camera();

        let dt = input.delta_time_secs;

        handle_input(
            &mut player,
            input.pressed_keys.iter(),
            input.released_keys.iter(),
        );

        let cursor_pos: Vector2<f32> = frame.renderer().window_to_world(input.cursor_pos);
        let angle = -(player.position.x - cursor_pos.x).atan2(player.position.y - cursor_pos.y);

        if input
            .pressed_keys
            .iter()
            .any(|(_, keycode)| matches!(keycode, VirtualKeyCode::Space))
        {
            projectiles.push(Projectile {
                position: player.position,
                velocity: [angle.sin(), -angle.cos()].into(),
            });
        }

        let mut wish_x = player.wish.x as f32;
        let mut wish_y = player.wish.y as f32;
        let wish_len = (wish_x * wish_x + wish_y * wish_y).sqrt();
        if wish_len != 0. {
            wish_x /= wish_len;
            wish_y /= wish_len;
            player.velocity.x += wish_x * dt;
            player.velocity.y += wish_y * dt;
            let len = (player.velocity.x * player.velocity.x
                + player.velocity.y * player.velocity.y)
                .sqrt();
            if len > 1. {
                player.velocity.x /= len;
                player.velocity.y /= len;
            }
        } else {
            player.velocity.x *= player.drag.powf(1. - dt);
            player.velocity.y *= player.drag.powf(1. - dt);
        }

        player.position.x += player.velocity.x * dt;
        player.position.y += player.velocity.y * dt;

        let mut asteroids_to_remove = vec![];

        for Projectile {
            position, velocity, ..
        } in projectiles.iter_mut()
        {
            position.x += velocity.x * dt * PROJECTILE_SPEED;
            position.y += velocity.y * dt * PROJECTILE_SPEED;

            frame
                .sprite(projectile_sprite)
                .layer(FOREGROUND)
                .pos([position.x, position.y, 0.])
                .scale(0.5)
                .draw();

            for (
                i,
                Asteroid {
                    position: asteroid_pos,
                    size,
                    ..
                },
            ) in asteroids.iter().enumerate()
            {
                let dx = position.x - asteroid_pos.x;
                let dy = position.y - asteroid_pos.y;

                let distance = (dx * dx + dy * dy).sqrt();
                if distance <= *size {
                    asteroids_to_remove.push(i);
                }
            }
        }

        for (removed_count, i) in asteroids_to_remove.into_iter().enumerate() {
            let Asteroid {
                position,
                size,
                rotor,
                ..
            } = asteroids.remove(i - removed_count);

            if size > 1. {
                asteroids.push(Asteroid {
                    position,
                    velocity: [(rotor + angle).sin(), (rotor + angle).cos()].into(),
                    size: size - 1.,
                    rotor: rotor / 2.,
                });

                asteroids.push(Asteroid {
                    position,
                    velocity: [(rotor - angle).sin(), (rotor - angle).cos()].into(),
                    size: size - 1.,
                    rotor: rotor * 2. + 170.,
                });
            }
        }

        for Asteroid {
            position,
            velocity,
            size: asteroid_size,
            rotor,
        } in asteroids.iter_mut()
        {
            position.x += velocity.x * dt;
            position.y += velocity.y * dt;

            *position = rect_wrap(*position, size, size * aspect_ratio, *asteroid_size);

            *rotor += dt * size;

            frame
                .sprite(asteroid_sprite)
                .layer(FOREGROUND)
                .pos([position.x, position.y, 0.])
                .scale(*asteroid_size)
                .rotate(ROCK_SPINOR * *rotor)
                .draw();
        }

        frame
            .sprite(cursor_sprite)
            .layer(FOREGROUND)
            .pos([cursor_pos.x, cursor_pos.y, 0.])
            .opacity(0.333)
            .scale(0.5)
            .draw();

        player.position = rect_wrap(player.position, size, size * aspect_ratio, 1.);

        frame
            .sprite(spaceship_sprite)
            .layer(FOREGROUND)
            .pos([player.position.x, player.position.y, 0.])
            .rotate(angle + PI)
            .draw();
    }
}

fn main() {
    let mut renderer = StandaloneRenderer::new("Tangerine Asteroids");

    let asteroid_texture =
        image::load_from_memory(include_bytes!("./assets/asteroid.png")).unwrap();
    let spaceship_texture =
        image::load_from_memory(include_bytes!("./assets/spaceship.png")).unwrap();
    let cursor_texture = image::load_from_memory(include_bytes!("./assets/cursor.png")).unwrap();
    let projectile_texture =
        image::load_from_memory(include_bytes!("./assets/projectile.png")).unwrap();

    let [asteroid_sprite, spaceship_sprite, cursor_sprite, projectile_sprite] = renderer
        .atlas()
        .add_sprite(asteroid_texture)
        .add_sprite(spaceship_texture)
        .add_sprite(cursor_texture)
        .add_sprite(projectile_texture)
        .finalize_and_repack();

    renderer.set_layer(BACKGROUND, -1);
    renderer.set_layer(FOREGROUND, 1);

    renderer.mutate_camera(|camera| camera.size = 8.);

    renderer.run_infallible(make_draw_callback(
        asteroid_sprite,
        spaceship_sprite,
        cursor_sprite,
        projectile_sprite,
    ));
}
