use crate::game::{
    ball::SpawnBallEvent,
    brick::{self, BrickComponent},
    paddle::{self, PlayerPaddleComponent},
    physics::ColliderComponent,
    render::SpriteComponent,
    transform::TransformComponent,
    Point2f, Vector2d, Vector2f, WORLD_UNIT_RATIO,
};
use gfx::{color::*, renderer::Transparency, sprite::SpriteRegion};
use nalgebra::Vector2;
use ncollide2d::shape::Cuboid;
use shrev::EventChannel;
use specs::prelude::*;

pub const PLAYER_DEFAULT_BALLS: u32 = 3;
pub const LEVEL_BRICKS_Y_OFFSET: f64 = 22.0;
pub const LEVEL_BRICKS_WIDTH: u32 = 10;
pub const LEVEL_BRICKS_HEIGHT: u32 = 5;

#[derive(Default)]
pub struct LevelState {
    pub score: u32,
    pub lives: u32,
    pub player_paddle_ent: Option<Entity>,
    pub load_level_event: Option<LoadLevelEvent>,
    pub level_width: u32,
    pub level_height: u32,
}

impl LevelState {
    pub fn new(
        level_width: u32,
        level_height: u32,
        load_level_event: LoadLevelEvent,
    ) -> LevelState {
        LevelState {
            score: 0,
            lives: 3,
            player_paddle_ent: None,
            load_level_event: Some(load_level_event),
            level_width,
            level_height,
        }
    }

    pub fn reset(&mut self, player_paddle_ent: Entity) {
        self.score = 0;
        self.lives = PLAYER_DEFAULT_BALLS;
        self.player_paddle_ent = Some(player_paddle_ent);
        self.load_level_event = None;
    }
}

#[derive(Clone, Copy)]
pub struct LoadLevelEvent;

pub fn load_level(world: &mut World) {
    println!("Loading level...");

    world.delete_all();

    let solid_collision_groups = ncollide2d::pipeline::CollisionGroups::new().with_membership(&[1]);

    let (level_width, level_height) = {
        let level = world.read_resource::<LevelState>();
        (level.level_width, level.level_height)
    };

    // Spawn player paddle
    let paddle_position = Vector2d::new(level_width as f64 / 2.0, level_height as f64 - 10.0);
    let player_paddle_ent = world
        .create_entity()
        .with(TransformComponent::new(
            paddle_position,
            Point2f::new(30.0, 16.0),
            Vector2f::new(paddle::PADDLE_SCALE_X, paddle::PADDLE_SCALE_Y),
        ))
        .with(ColliderComponent::new(
            Cuboid::new(Vector2::new(
                (paddle::PADDLE_HIT_BOX_WIDTH / 2.0) * WORLD_UNIT_RATIO,
                (paddle::PADDLE_HIT_BOX_HEIGHT / 2.0) * WORLD_UNIT_RATIO,
            )),
            Vector2::zeros(),
            solid_collision_groups,
            1.0,
        ))
        .with(PlayerPaddleComponent::new(level_width))
        .with(SpriteComponent {
            color: COLOR_WHITE,
            spritesheet_tex_id: 2,
            region: SpriteRegion {
                x: 0,
                y: 0,
                w: paddle::PADDLE_SPRITE_WIDTH,
                h: paddle::PADDLE_SPRITE_HEIGHT,
            },
            layer: 1,
            transparency: Transparency::Opaque,
        })
        .build();

    // Spawn bricks
    for y in 0..LEVEL_BRICKS_HEIGHT {
        for x in 0..LEVEL_BRICKS_WIDTH {
            let position = Vector2d::new(
                x as f64 * brick::BRICK_SPRITE_WIDTH as f64,
                LEVEL_BRICKS_Y_OFFSET + (y as f64 * brick::BRICK_SPRITE_HEIGHT as f64),
            );

            world
                .create_entity()
                .with(TransformComponent::new(
                    position,
                    Point2f::origin(),
                    Vector2f::new(1.0, 1.0),
                ))
                .with(ColliderComponent::new(
                    Cuboid::new(Vector2::new(0.5, 0.25)),
                    Vector2::new(16.0, 8.0),
                    solid_collision_groups,
                    0.0,
                ))
                .with(BrickComponent::new(brick::BRICK_DEFAULT_HP))
                .with(SpriteComponent {
                    color: COLOR_WHITE,
                    spritesheet_tex_id: 2,
                    region: SpriteRegion {
                        x: 96,
                        y: 0,
                        w: brick::BRICK_SPRITE_WIDTH,
                        h: brick::BRICK_SPRITE_HEIGHT,
                    },
                    layer: 2,
                    transparency: Transparency::Opaque,
                })
                .build();
        }
    }

    // Spawn initial ball
    world
        .write_resource::<EventChannel<SpawnBallEvent>>()
        .single_write(SpawnBallEvent {
            position: Vector2d::new(level_width as f64 / 2.0, level_height as f64 / 2.0),
            linear_velocity: Vector2d::new(2.5, -2.5),
            owning_paddle_ent: Some(player_paddle_ent),
        });

    // Spawn Left wall
    world
        .create_entity()
        .with(TransformComponent {
            position: Vector2d::new(-20.0, 0.0),
            ..Default::default()
        })
        .with(ColliderComponent::new(
            Cuboid::new(Vector2::new(20.0 * WORLD_UNIT_RATIO, 50.0)),
            Vector2::zeros(),
            solid_collision_groups,
            1.0,
        ))
        .build();

    // Spawn Top wall
    world
        .create_entity()
        .with(TransformComponent {
            position: Vector2d::new(0.0, -20.0),
            ..Default::default()
        })
        .with(ColliderComponent::new(
            Cuboid::new(Vector2::new(50.0, 20.0 * WORLD_UNIT_RATIO)),
            Vector2::zeros(),
            solid_collision_groups,
            1.0,
        ))
        .build();

    // Spawn Right wall
    world
        .create_entity()
        .with(TransformComponent {
            position: Vector2d::new(level_width as f64 + 20.0, 0.0),
            ..Default::default()
        })
        .with(ColliderComponent::new(
            Cuboid::new(Vector2::new(20.0 * WORLD_UNIT_RATIO, 50.0)),
            Vector2::zeros(),
            solid_collision_groups,
            1.0,
        ))
        .build();

    world
        .write_resource::<LevelState>()
        .reset(player_paddle_ent);

    world.maintain();
}
