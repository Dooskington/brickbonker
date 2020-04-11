mod game;

use game::{physics::PhysicsState, render::RenderState, GameState};

use gfx::{
    color::*,
    image::*,
    input::InputState,
    renderer::*,
    sprite::*,
    texture::*,
    window::{self, *},
};
use nalgebra::{Point2, Vector2};
use specs::prelude::*;

fn main() {
    let window_title: &str = "Brickbonker";
    let window_width: u32 = 320;
    let window_height: u32 = 240;
    let render_scale: f32 = 2.0;
    let state = GameState::new(window_width, window_height);

    window::run(
        window_title,
        window_width,
        window_height,
        render_scale,
        state,
        move |_game, renderer| {
            import_texture(1, "res/textures/costanza.png", renderer);
            import_texture(2, "res/textures/sprites.png", renderer);
            import_texture(3, "res/textures/font.png", renderer);
        },
        move |game, _window, input, dt| {
            game.world.insert::<InputState>(input.clone());
            game.world.insert::<DeltaTime>(dt);

            game.world.write_resource::<RenderState>().clear_commands();
            game.tick_dispatcher.dispatch(&mut game.world);
            game.physics_dispatcher.dispatch(&mut game.world);
            game.world.maintain();
        },
        move |game, _ticks, lerp, window, renderer| {
            game.world.write_resource::<PhysicsState>().lerp = lerp;
            // Process commands into batches and send to the renderer
            let mut commands = game.world.write_resource::<RenderState>().commands();

            let msg = format!("{}", window.fps);
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
                        x: 4.0 + (i as f32 * 4.0),
                        y: 4.0,
                        origin: Point2::origin(),
                        scale: Vector2::new(0.5, 0.5),
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
            renderer.render(window.dpi_scale_factor, batches);
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
