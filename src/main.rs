mod game;
mod physics;

use crate::game::{GameState, RenderCommander};
use gfx::{
    color::*,
    image::*,
    input::{InputState, VirtualKeyCode},
    renderer::*,
    sprite::*,
    texture::*,
    window::{self, *},
};
use nalgebra::{Point2, Vector2};
use physics::PhysicsState;
use specs::prelude::*;

fn main() {
    let window_title: &str = "Brickbreaker";
    let window_width: u32 = 640;
    let window_height: u32 = 480;
    let state = GameState::new();

    // Need to pass in a resolution and scale
    // then create a window with the scale
    // and projection with the original resolution?

    window::run(
        window_title,
        window_width,
        window_height,
        state,
        move |game, renderer| {
            import_texture(1, "res/textures/costanza.png", renderer);
            import_texture(2, "res/textures/sprites.png", renderer);
            import_texture(3, "res/textures/font.png", renderer);
        },
        move |game, _window, input, dt| {
            game.world.insert::<InputState>(input.clone());
            game.world.insert::<DeltaTime>(dt);

            game.world
                .write_resource::<RenderCommander>()
                .clear_commands();
            game.tick_dispatcher.dispatch(&mut game.world);
            game.physics_dispatcher.dispatch(&mut game.world);
            game.world.maintain();
        },
        move |game, ticks, lerp, window, renderer| {
            game.world.write_resource::<PhysicsState>().lerp = lerp;
            // Process commands into batches and send to the renderer
            let mut commands = game.world.write_resource::<RenderCommander>().commands();

            let msg = format!("FPS: {}, Ticks: {}", window.fps, ticks);
            for (i, c) in msg.chars().enumerate() {
                let cols: u32 = 16;
                let ascii: u8 = c as u8;
                let sprite_col: u32 = ascii as u32 % cols;
                let sprite_row: u32 = ascii as u32 / cols;
                commands.push(gfx::renderer::RenderCommand {
                    transparency: Transparency::Transparent,
                    shader_program_id: 1,
                    tex_id: 3,
                    layer: 0,
                    data: Renderable::Sprite {
                        x: 32.0 + (i as f32 * 8.0),
                        y: 32.0,
                        origin: Point2::origin(),
                        scale: Vector2::new(1.0, 1.0),
                        color: COLOR_WHITE,
                        region: SpriteRegion {
                            x: sprite_col * 8,
                            y: sprite_row * 16,
                            w: 8,
                            h: 16,
                        },
                    },
                });
            }

            let batches = renderer.process_commands(commands);
            renderer.render(window.scale_factor, batches);
        },
    );
}

fn import_texture(id: u16, path: &str, renderer: &mut Renderer) -> Texture {
    let image: RgbaImage = gfx::image::open(path)
        .expect(&format!("Failed to open image {}!", path))
        .to_rgba();

    let width: u32 = image.width();
    let height: u32 = image.height();
    let pixels: Vec<u8> = image.into_raw();
    renderer.create_gpu_texture(id, width, height, &pixels);

    Texture::new(id, width, height, pixels)
}
