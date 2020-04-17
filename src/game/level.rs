use crate::game::{
    ball::SpawnBallEvent,
    brick::{self, BrickComponent, BrickType},
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
use std::collections::HashMap;

pub const PLAYER_DEFAULT_BALLS: u32 = 99;
pub const LEVEL_BRICKS_Y_OFFSET: f64 = 32.0;
pub const LEVEL_BRICKS_GRID_SIZE: u32 = 16;

#[derive(Default)]
pub struct LevelState {
    pub level: LevelAssetId,
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
            level: 0,
            score: 0,
            lives: 3,
            player_paddle_ent: None,
            load_level_event: Some(load_level_event),
            level_width,
            level_height,
        }
    }

    pub fn setup(&mut self, level: LevelAssetId, player_paddle_ent: Entity) {
        self.level = level;
        self.score = 0;
        self.lives = PLAYER_DEFAULT_BALLS;
        self.player_paddle_ent = Some(player_paddle_ent);
        self.load_level_event = None;
    }
}

pub type LevelAssetId = u32;

#[derive(Clone, Copy)]
pub struct LoadLevelEvent {
    pub level: LevelAssetId,
}

#[derive(Clone)]
pub struct LevelAsset {
    pub id: LevelAssetId,
    pub bricks: Vec<BrickType>,
}

pub struct LevelAssetDb {
    levels: HashMap<LevelAssetId, LevelAsset>,
}

impl LevelAssetDb {
    pub fn new() -> Self {
        LevelAssetDb {
            levels: HashMap::new(),
        }
    }

    pub fn import_folder(&mut self, path: &str) -> std::io::Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.is_file() {
                let id = file_path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .parse::<u32>()
                    .unwrap();
                let mut file_string = std::fs::read_to_string(file_path)?;
                file_string = file_string.replace("\n", "");
                file_string.retain(|c| !c.is_whitespace());

                let mut bricks = Vec::new();
                for c in file_string.chars() {
                    let brick_type = match c {
                        '0' => BrickType::Grey,
                        '1' => BrickType::Green,
                        '2' => BrickType::Blue,
                        '3' => BrickType::Red,
                        '4' => BrickType::Purple,
                        _ => BrickType::Air,
                    };

                    bricks.push(brick_type);
                }

                let level = LevelAsset { id, bricks };

                self.levels.insert(id, level);
            }
        }

        Ok(())
    }

    pub fn level(&self, id: LevelAssetId) -> Option<LevelAsset> {
        self.levels.get(&id).cloned()
    }
}

pub fn build_level(world: &mut World, level: LevelAsset) {
    println!("Loading level...");

    world.delete_all();

    let solid_collision_groups = ncollide2d::pipeline::CollisionGroups::new().with_membership(&[1]);

    let (level_width, level_height) = {
        let level_state = world.read_resource::<LevelState>();
        (level_state.level_width, level_state.level_height)
    };

    // Spawn player paddle
    let paddle_sprite = SpriteRegion {
        x: 0,
        y: 0,
        w: paddle::PADDLE_SPRITE_WIDTH,
        h: paddle::PADDLE_SPRITE_HEIGHT,
    };
    let paddle_position = Vector2d::new(level_width as f64 / 2.0, level_height as f64 - 10.0);
    let player_paddle_ent = world
        .create_entity()
        .with(TransformComponent::new(
            paddle_position,
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
        .with(SpriteComponent::new(
            paddle_sprite,
            2,
            Point2f::new(0.5, 0.5),
            COLOR_WHITE,
            1,
            Transparency::Opaque,
        ))
        .build();

    // Spawn bricks
    for y in 0..LEVEL_BRICKS_GRID_SIZE {
        for x in 0..LEVEL_BRICKS_GRID_SIZE {
            let position = Vector2d::new(
                x as f64 * brick::BRICK_SPRITE_WIDTH as f64,
                LEVEL_BRICKS_Y_OFFSET + (y as f64 * brick::BRICK_SPRITE_HEIGHT as f64),
            );

            let idx = (y * LEVEL_BRICKS_GRID_SIZE) + x;
            let brick_type = level.bricks[idx as usize];
            if brick_type == BrickType::Air {
                continue;
            }

            let brick_hp = match brick_type {
                BrickType::Grey => 0,
                BrickType::Green => 1,
                BrickType::Blue => 2,
                BrickType::Red => 3,
                BrickType::Purple => 4,
                _ => brick::BRICK_DEFAULT_HP,
            };

            let brick_sprite = SpriteRegion {
                x: 96,
                y: 0,
                w: brick::BRICK_SPRITE_WIDTH,
                h: brick::BRICK_SPRITE_HEIGHT,
            };

            world
                .create_entity()
                .with(TransformComponent::new(position, Vector2f::new(1.0, 1.0)))
                .with(ColliderComponent::new(
                    Cuboid::new(Vector2::new(
                        (brick::BRICK_SPRITE_WIDTH as f64 / 2.0) * WORLD_UNIT_RATIO,
                        (brick::BRICK_SPRITE_HEIGHT as f64 / 2.0) * WORLD_UNIT_RATIO,
                    )),
                    Vector2::new(
                        brick::BRICK_SPRITE_WIDTH as f64 / 2.0,
                        brick::BRICK_SPRITE_HEIGHT as f64 / 2.0,
                    ),
                    solid_collision_groups,
                    0.0,
                ))
                .with(BrickComponent::new(brick_hp))
                .with(SpriteComponent::new(
                    brick_sprite,
                    2,
                    Point2f::origin(),
                    COLOR_WHITE,
                    2,
                    Transparency::Opaque,
                ))
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
        .setup(level.id, player_paddle_ent);

    world.maintain();
}
