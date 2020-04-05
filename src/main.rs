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
    window,
};
use physics::PhysicsState;
use specs::prelude::*;

fn main() {
    let window_title: &str = "Brickbreaker";
    let window_width: u32 = 640;
    let window_height: u32 = 480;
    let state = GameState::new();

    window::run(
        window_title,
        window_width,
        window_height,
        state,
        move |game, renderer| {
            import_texture(1, "res/textures/costanza.png", renderer);
            import_texture(2, "res/textures/sprites.png", renderer);
        },
        move |game, input| {
            game.world.insert::<InputState>(input.clone());

            game.world
                .write_resource::<RenderCommander>()
                .clear_commands();
            game.tick_dispatcher.dispatch(&mut game.world);
            game.physics_dispatcher.dispatch(&mut game.world);
            game.world.maintain();
        },
        move |game, ticks, lerp, renderer| {
            game.world.write_resource::<PhysicsState>().lerp = lerp;
            // Process commands into batches and send to the renderer
            let commands = game.world.write_resource::<RenderCommander>().commands();
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
