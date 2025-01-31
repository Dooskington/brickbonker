pub mod audio;
pub mod ball;
pub mod brick;
pub mod level;
pub mod paddle;
pub mod physics;
pub mod render;
pub mod transform;

use audio::AudioAssetDb;
use ball::{BallSystem, SpawnBallSystem};
use brick::BrickSystem;
use level::{LevelState, LoadLevelEvent};
use paddle::PlayerPaddleSystem;
use physics::{
    ColliderSendPhysicsSystem, PhysicsState, RigidbodyReceivePhysicsSystem,
    RigidbodySendPhysicsSystem, WorldStepPhysicsSystem,
};
use render::{RenderState, SpriteRenderSystem};
use specs::prelude::*;
use transform::TransformComponent;

pub type Vector2f = nalgebra::Vector2<f32>;
pub type Vector2d = nalgebra::Vector2<f64>;
pub type Point2f = nalgebra::Point2<f32>;
pub type Point2d = nalgebra::Point2<f64>;

pub const PIXELS_PER_WORLD_UNIT: u32 = 32;
pub const WORLD_UNIT_RATIO: f64 = (1.0 / PIXELS_PER_WORLD_UNIT as f64);

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
            .with(BrickSystem::default(), "brick", &[])
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

        // Resources
        world.insert(RenderState::new());
        world.insert(LevelState::new(width, height, LoadLevelEvent));
        world.insert(PhysicsState::new());
        world.insert(AudioAssetDb::new());

        GameState {
            world,
            tick_dispatcher,
            physics_dispatcher,
        }
    }
}
