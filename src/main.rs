mod game;

use crate::game::GameState;
use gfx::{color::*, image::*, input::InputState, renderer::*, sprite::*, texture::*, window};
use rand::Rng;
use std::rc::Rc;
use time::*;
use nalgebra::*;
use std::cell::RefCell;

fn main() {
    let window_title: &str = "Brickbreaker";
    let window_width: u32 = 800;
    let window_height: u32 = 600;

    let state = GameState::new();

    let paddle_sprite = SpriteRegion {
        x: 0,
        y: 0,
        w: 64,
        h: 32,
    };

    let ball_sprite = SpriteRegion {
        x: 64,
        y: 0,
        w: 32,
        h: 32,
    };

    let brick_sprite = SpriteRegion {
        x: 96,
        y: 0,
        w: 32,
        h: 32,
    };

    window::run(
        window_title,
        window_width,
        window_height,
        state,
        move |renderer, state| {
            import_texture(1, "res/textures/costanza.png", renderer);
            import_texture(2, "res/textures/sprites.png", renderer);

            println!("Initialized!");
        },
        move |state| {
        },
        move |t, renderer, state| {
            let mut commands: Vec<RenderCommand> = Vec::new();

            commands.push(gfx::renderer::RenderCommand {
                transparency: Transparency::Opaque,
                shader_program_id: 1,
                tex_id: 1,
                layer: 1,
                data: Renderable::Quad {
                    bl: (16.0, 116.0),
                    br: (116.0, 116.0),
                    tl: (16.0, 16.0),
                    tr: (116.0, 16.0),
                    color: COLOR_WHITE,
                },
            });

            commands.push(gfx::renderer::RenderCommand {
                transparency: Transparency::Opaque,
                shader_program_id: 1,
                tex_id: 2,
                layer: 1,
                data: Renderable::Sprite {
                    x: 128.0,
                    y: 128.0,
                    w: 64.0,
                    h: 32.0,
                    color: COLOR_WHITE,
                    region: paddle_sprite,
                },
            });

            commands.push(gfx::renderer::RenderCommand {
                transparency: Transparency::Opaque,
                shader_program_id: 1,
                tex_id: 2,
                layer: 1,
                data: Renderable::Sprite {
                    x: 200.0,
                    y: 128.0,
                    w: 32.0,
                    h: 32.0,
                    color: COLOR_WHITE,
                    region: ball_sprite,
                },
            });

            commands.push(gfx::renderer::RenderCommand {
                transparency: Transparency::Opaque,
                shader_program_id: 1,
                tex_id: 2,
                layer: 1,
                data: Renderable::Sprite {
                    x: 250.0,
                    y: 128.0,
                    w: 32.0,
                    h: 32.0,
                    color: COLOR_WHITE,
                    region: brick_sprite,
                },
            });

            // Process commands into batches and send to the renderer
            let batches = renderer.process_commands(commands);
            renderer.render(batches);
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
