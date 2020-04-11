pub mod audio;
pub mod ball;
pub mod brick;
pub mod paddle;
pub mod physics;
pub mod render;
pub mod transform;

use ball::{BallSystem, SpawnBallEvent, SpawnBallSystem};
use brick::BreakableComponent;
use gfx::{color::*, sprite::SpriteRegion};
use nalgebra::Vector2;
use ncollide2d::shape::Cuboid;
use nphysics2d::object::BodyStatus;
use paddle::{PlayerPaddleComponent, PlayerPaddleSystem};
use physics::{
    ColliderComponent, ColliderSendPhysicsSystem, PhysicsState, RigidbodyComponent,
    RigidbodyReceivePhysicsSystem, RigidbodySendPhysicsSystem, WorldStepPhysicsSystem,
};
use render::{RenderState, SpriteComponent, SpriteRenderSystem};
use shrev::EventChannel;
use specs::prelude::*;
use transform::TransformComponent;

pub type Vector2f = nalgebra::Vector2<f32>;
pub type Vector2d = nalgebra::Vector2<f64>;
pub type Point2f = nalgebra::Point2<f32>;
pub type Point2d = nalgebra::Point2<f64>;

pub const PIXELS_PER_WORLD_UNIT: u32 = 32;
pub const WORLD_UNIT_RATIO: f64 = (1.0 / PIXELS_PER_WORLD_UNIT as f64);

const PADDLE_SPRITE_WIDTH: u32 = 64;
const PADDLE_SPRITE_HEIGHT: u32 = 32;
const PADDLE_SCALE_X: f32 = 1.0;
const PADDLE_SCALE_Y: f32 = 1.0;

pub struct GameState<'a, 'b> {
    pub world: World,
    pub tick_dispatcher: Dispatcher<'a, 'b>,
    pub physics_dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> GameState<'a, 'b> {
    pub fn new(width: u32, height: u32) -> GameState<'a, 'b> {
        let mut world = World::new();

        let mut tick_dispatcher = DispatcherBuilder::new()
            .with(PlayerPaddleSystem, "player_paddle", &[])
            .with(BallSystem::default(), "ball", &[])
            .with_thread_local(SpawnBallSystem::default())
            .with_thread_local(SpriteRenderSystem::default())
            .build();

        tick_dispatcher.setup(&mut world);

        let mut physics_dispatcher = DispatcherBuilder::new()
            .with_thread_local(RigidbodySendPhysicsSystem::default())
            .with_thread_local(ColliderSendPhysicsSystem::default())
            .with_thread_local(WorldStepPhysicsSystem)
            .with_thread_local(RigidbodyReceivePhysicsSystem)
            .build();

        physics_dispatcher.setup(&mut world);

        let solid_collision_groups =
            ncollide2d::pipeline::CollisionGroups::new().with_membership(&[1]);

        // Spawn player paddle
        let paddle_position = Vector2d::new(width as f64 / 2.0, height as f64 - 10.0);
        let paddle_ent = world
            .create_entity()
            .with(TransformComponent::new(
                paddle_position,
                Point2f::new(30.0, 16.0),
                Vector2f::new(PADDLE_SCALE_X, PADDLE_SCALE_Y),
            ))
            .with(ColliderComponent::new(
                Cuboid::new(Vector2::new(
                    (paddle::PADDLE_HIT_BOX_WIDTH / 2.0) * WORLD_UNIT_RATIO,
                    (paddle::PADDLE_HIT_BOX_HEIGHT / 2.0) * WORLD_UNIT_RATIO,
                )),
                Vector2::zeros(),
                solid_collision_groups,
                0.0,
            ))
            .with(PlayerPaddleComponent::new(width))
            .with(SpriteComponent {
                color: COLOR_WHITE,
                spritesheet_tex_id: 2,
                region: SpriteRegion {
                    x: 0,
                    y: 0,
                    w: PADDLE_SPRITE_WIDTH,
                    h: PADDLE_SPRITE_HEIGHT,
                },
                layer: 1,
            })
            .build();

        // Spawn initial ball
        world
            .write_resource::<EventChannel<SpawnBallEvent>>()
            .single_write(SpawnBallEvent {
                position: Vector2d::new(width as f64 / 2.0, height as f64 / 2.0),
                linear_velocity: Vector2d::new(2.5, -2.5),
                owning_paddle_ent: Some(paddle_ent),
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
                position: Vector2d::new(width as f64 + 20.0, 0.0),
                ..Default::default()
            })
            .with(ColliderComponent::new(
                Cuboid::new(Vector2::new(20.0 * WORLD_UNIT_RATIO, 50.0)),
                Vector2::zeros(),
                solid_collision_groups,
                1.0,
            ))
            .build();

        // Resources
        world.insert(RenderState::new());
        world.insert(LevelState {
            level: 1,
            player_paddle_ent: Some(paddle_ent),
        });
        world.insert(PhysicsState::new());

        GameState {
            world,
            tick_dispatcher,
            physics_dispatcher,
        }
    }
}

#[derive(Default)]
pub struct LevelState {
    pub level: i32,
    pub player_paddle_ent: Option<Entity>,
}
