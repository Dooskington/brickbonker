mod game;

use game::{
    audio::{self, AudioAssetId, AudioAssetDb},
    level::{self, LevelState},
    physics::PhysicsState,
    render::RenderState,
    GameState,
};

use gfx::{
    color::*,
    image::*,
    input::InputState,
    renderer::*,
    texture::*,
    window::{self, *},
};
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
        move |game, renderer| {
            import_texture(1, "res/textures/costanza.png", renderer);
            import_texture(2, "res/textures/sprites.png", renderer);
            import_texture(3, "res/textures/font.png", renderer);
            import_texture(4, "res/textures/bg.png", renderer);

            // Import audio assets (music and sound effects)
            {
                let mut audio_db = game.world.write_resource::<AudioAssetDb>();
                audio_db.import(AudioAssetId::MusicBackground, "res/audio/tha-bounce-life.wav").unwrap();
                audio_db.import(AudioAssetId::SfxBallBounce0, "res/audio/ball-bounce-0.wav").unwrap();
                audio_db.import(AudioAssetId::SfxBallBounce1, "res/audio/ball-bounce-1.wav").unwrap();
                audio_db.import(AudioAssetId::SfxBallWallHit0, "res/audio/ball-wall-hit-0.wav").unwrap();
                audio_db.import(AudioAssetId::SfxBallWallHit1, "res/audio/ball-wall-hit-1.wav").unwrap();
                audio_db.import(AudioAssetId::SfxBrickBreak0, "res/audio/brick-break-0.wav").unwrap();
                audio_db.import(AudioAssetId::SfxBrickBreak1, "res/audio/brick-break-1.wav").unwrap();
                audio_db.import(AudioAssetId::SfxBallDeath0, "res/audio/ball-death-0.wav").unwrap();

                // Start playing the bg music right away
                audio::play(AudioAssetId::MusicBackground, &audio_db, true);
            }

        },
        move |game, _window, input, dt| {
            game.world.insert::<InputState>(input.clone());
            game.world.insert::<DeltaTime>(dt);

            // Handle any level loads
            let load_level_pending = game
                .world
                .read_resource::<LevelState>()
                .load_level_event
                .is_some();
            if load_level_pending {
                level::load_level(&mut game.world);
            }

            game.world.write_resource::<RenderState>().clear_commands();
            game.tick_dispatcher.dispatch(&mut game.world);
            game.physics_dispatcher.dispatch(&mut game.world);

            game.world.maintain();
        },
        move |game, _ticks, lerp, window, renderer| {
            game.world.write_resource::<PhysicsState>().lerp = lerp;

            let mut render = game.world.write_resource::<RenderState>();

            // FPS text
            let msg = format!("FPS: {}", window.fps);
            render.bind_color(COLOR_WHITE);
            render.bind_layer(0);
            render.bind_transparency(Transparency::Transparent);
            render.bind_texture(3);
            let fps_text_x = window_width as f32 - (msg.len() as f32 * 4.0) - 2.0;
            render.text(fps_text_x, 2.0, 8, 16, 0.5, &msg);

            let (score, balls, is_game_over) = {
                let level = game.world.read_resource::<LevelState>();
                (level.score, level.lives, level.lives == 0)
            };

            // Score text
            let msg = format!("Score: {}", score);
            render.bind_color(if is_game_over {
                COLOR_GREEN
            } else {
                COLOR_WHITE
            });
            render.text(2.0, 2.0, 8, 16, 0.5, &msg);

            // Balls text
            let msg = format!("Balls: {}", balls);
            render.bind_color(COLOR_WHITE);
            render.text(2.0, 10.0, 8, 16, 0.5, &msg);
            if is_game_over {
                // Game Over text
                let game_over_text_y = window_height as f32 - 22.0;
                render.bind_color(COLOR_RED);
                render.text(2.0, game_over_text_y, 8, 16, 0.75, &format!("Game Over!"));

                // Restart text
                let restart_text_y = window_height as f32 - 10.0;
                render.bind_color(COLOR_WHITE);
                render.text(
                    2.0,
                    restart_text_y,
                    8,
                    16,
                    0.5,
                    &format!("Press 'R' to start a new game."),
                );
            }

            // Background
            render.bind_color(COLOR_WHITE);
            render.bind_layer(0);
            render.bind_transparency(Transparency::Opaque);
            render.bind_texture(4);
            render.textured_quad((0.0, 400.0), (400.0, 400.0), (0.0, 0.0), (400.0, 0.0));

            // Process commands into batches and send to the renderer
            let batches = renderer.process_commands(render.commands());
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
