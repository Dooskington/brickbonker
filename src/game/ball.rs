use crate::game::{
    audio::{self, AudioAssetDb, AudioAssetId},
    brick::BrickComponent,
    paddle::{PlayerPaddleComponent, PADDLE_HIT_BOX_WIDTH},
    physics::{ColliderComponent, CollisionEvent, RigidbodyComponent},
    render::SpriteComponent,
    transform::TransformComponent,
    LevelState, Point2f, Vector2d, Vector2f,
};
use gfx::{color::*, renderer::Transparency, sprite::SpriteRegion};
use nalgebra::Vector2;
use ncollide2d::shape::Ball;
use nphysics2d::{math::Velocity, object::BodyStatus};
use shrev::EventChannel;
use specs::prelude::*;

pub const BALL_COLLIDER_RADIUS: f64 = 2.75;
pub const BALL_MAX_LINEAR_VELOCITY: f64 = 12.0;
pub const BALL_DEFAULT_FORCE: f64 = 5.0;

#[derive(Clone, Debug)]
pub struct SpawnBallEvent {
    pub position: Vector2d,
    pub linear_velocity: Vector2d,
    pub owning_paddle_ent: Option<Entity>,
}

#[derive(Debug)]
pub struct BallComponent {
    pub last_pos: Point2f,
    pub holding_paddle_ent: Option<Entity>,
    pub velocity: Velocity<f64>,
}

impl BallComponent {
    pub fn new(linear_velocity: Vector2d, holding_paddle_ent: Option<Entity>) -> Self {
        BallComponent {
            last_pos: Point2f::origin(),
            velocity: Velocity::new(linear_velocity, 0.0),
            holding_paddle_ent,
        }
    }
}

impl Component for BallComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct BallSystem {
    collision_event_reader: Option<ReaderId<CollisionEvent>>,
}

impl<'a> System<'a> for BallSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, LevelState>,
        ReadExpect<'a, AudioAssetDb>,
        Read<'a, EventChannel<CollisionEvent>>,
        Write<'a, EventChannel<SpawnBallEvent>>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, BallComponent>,
        ReadStorage<'a, BrickComponent>,
        ReadStorage<'a, PlayerPaddleComponent>,
        WriteStorage<'a, RigidbodyComponent>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.collision_event_reader = Some(
            world
                .fetch_mut::<EventChannel<CollisionEvent>>()
                .register_reader(),
        );
    }

    fn run(
        &mut self,
        (
            ents,
            mut level,
            audio_db,
            collision_events,
            mut spawn_ball_events,
            mut transforms,
            mut balls,
            bricks,
            paddles,
            mut rigidbodies,
        ): Self::SystemData,
    ) {
        let mut balls_bounced_this_tick: BitSet = BitSet::new();
        for event in collision_events.read(&mut self.collision_event_reader.as_mut().unwrap()) {
            // Get the entities involved in the event, ignoring it entirely if either of them are not an entity
            let (entity_a, entity_b) = {
                if event.entity_a.is_none() || event.entity_b.is_none() {
                    continue;
                }

                (event.entity_a.unwrap(), event.entity_b.unwrap())
            };

            if let Some(ball) = balls.get_mut(entity_a) {
                if let Some(_) = paddles.get(entity_b) {
                    let paddle_transform = transforms.get(entity_b).unwrap();
                    let hit_x = match event.collision_point {
                        Some(p) => p.x,
                        None => {
                            println!("Ball collision had no collision_point! ball ent = {}, other ent = {}", entity_a.id(), entity_b.id());

                            // If there was no concrete collision point calculated, just use the balls current x position
                            let ball_transform = transforms.get(entity_a).unwrap();
                            ball_transform.position.x
                        }
                    };

                    // Get the x hit value, relative to the paddle hit box width. -1.0 means the ball hit the far left side of the paddle, while 1.0 means it hit the far right.
                    let hit_x_ratio =
                        (hit_x - paddle_transform.position.x) / (PADDLE_HIT_BOX_WIDTH / 2.0);

                    let mut vel = ball.velocity.linear;
                    vel.y = ((vel.x.abs() * 0.25) + vel.y) * -0.97;
                    vel.x = hit_x_ratio * BALL_DEFAULT_FORCE;

                    vel = vel.normalize()
                        * nalgebra::clamp(vel.magnitude(), 0.0, BALL_MAX_LINEAR_VELOCITY);
                    ball.velocity = Velocity::new(vel, 0.0);
                    println!("reflected off paddle: {:?}", ball.velocity);

                    // Pick and play one of the ball paddle bounce audio clips
                    let clip_id = {
                        use rand::Rng;
                        let roll: f32 = rand::thread_rng().gen();

                        if roll <= 0.5 {
                            AudioAssetId::SfxBallBounce0
                        } else {
                            AudioAssetId::SfxBallBounce1
                        }
                    };

                    audio::play(clip_id, &audio_db, false);

                    continue;
                }

                if let Some(normal) = event.normal {
                    let normal = -normal;
                    let normal = if (normal.x > 0.1) && (normal.y > 0.1) {
                        if normal.x > normal.y {
                            Vector2d::new(1.0, 0.0)
                        } else {
                            Vector2d::new(0.0, 1.0)
                        }
                    } else if (normal.x > 0.1) && (normal.y < 0.1) {
                        if normal.x > normal.y.abs() {
                            Vector2d::new(1.0, 0.0)
                        } else {
                            Vector2d::new(0.0, -1.0)
                        }
                    } else if (normal.x < 0.1) && (normal.y > 0.1) {
                        if normal.x.abs() > normal.y {
                            Vector2d::new(-1.0, 0.0)
                        } else {
                            Vector2d::new(0.0, 1.0)
                        }
                    } else if (normal.x < 0.1) && (normal.y < 0.1) {
                        if normal.x.abs() > normal.y.abs() {
                            Vector2d::new(-1.0, 0.0)
                        } else {
                            Vector2d::new(0.0, -1.0)
                        }
                    } else {
                        normal
                    };

                    let ent_b_is_brick = bricks.get(entity_b).is_some();
                    // If the ball already bounced this tick, and this is a brick, just ignore it
                    if ent_b_is_brick {
                        if balls_bounced_this_tick.contains(entity_a.id()) {
                            continue;
                        }

                        balls_bounced_this_tick.add(entity_a.id());
                    }

                    let vel = ball.velocity;
                    let dot = vel.linear.dot(&normal);

                    let mut reflected_vel = (vel.linear - (2.0 * dot) * normal) * 1.01;
                    reflected_vel = reflected_vel.normalize()
                        * nalgebra::clamp(reflected_vel.magnitude(), 0.0, BALL_MAX_LINEAR_VELOCITY);
                    ball.velocity = Velocity::new(reflected_vel, vel.angular);

                    println!(
                        "reflected off wall/brick: {:?}, normal was {:?}",
                        ball.velocity, normal
                    );

                    // Pick and play one of the ball hit audio clips
                    let clip_id = {
                        use rand::Rng;
                        let roll: f32 = rand::thread_rng().gen();

                        if roll <= 0.5 {
                            AudioAssetId::SfxBallWallHit0
                        } else {
                            AudioAssetId::SfxBallWallHit1
                        }
                    };

                    audio::play(clip_id, &audio_db, false);
                } else {
                    println!(
                        "Ball collision had no normal! ball ent = {}, other ent = {}",
                        entity_a.id(),
                        entity_b.id()
                    );
                }
            }
        }

        for (ent, mut transform, rigidbody, ball) in
            (&ents, &mut transforms, &mut rigidbodies, &mut balls).join()
        {
            if let Some(holding_paddle_ent) = ball.holding_paddle_ent {
                let paddle = paddles.get(holding_paddle_ent).unwrap();
                transform.position = paddle.held_ball_position;
                rigidbody.status = BodyStatus::Disabled;
                continue;
            }

            // If the ball was bounced this tick, send it back to where it was last tick just to avoid any double collisions or such issues
            if balls_bounced_this_tick.contains(ent.id()) {
                //transform.position = transform.last_position;
            }

            // Directly set the ball velocity every tick to keep the physics engine from affecting it
            rigidbody.status = BodyStatus::Dynamic;
            rigidbody.velocity = ball.velocity;

            // TODO replace this with a sensor collider?
            if transform.position.y > (level.level_height as f64 - 5.0) {
                ents.delete(ent).expect("Failed to delete ball ent!");

                audio::play(AudioAssetId::SfxBallDeath0, &audio_db, false);

                level.lives -= 1;
                println!("{} balls remaining.", level.lives);
                if level.lives == 0 {
                    println!("Game over!");
                } else {
                    // Spawn another ball
                    spawn_ball_events.single_write(SpawnBallEvent {
                        position: Vector2d::zeros(),
                        linear_velocity: Vector2d::zeros(),
                        owning_paddle_ent: level.player_paddle_ent,
                    });
                }

                continue;
            }
        }
    }
}

#[derive(Default)]
pub struct SpawnBallSystem {
    spawn_ball_event_reader: Option<ReaderId<SpawnBallEvent>>,
}

impl<'a> System<'a> for SpawnBallSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        Read<'a, EventChannel<SpawnBallEvent>>,
        WriteStorage<'a, PlayerPaddleComponent>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.spawn_ball_event_reader = Some(
            world
                .fetch_mut::<EventChannel<SpawnBallEvent>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (ents, lazy_updater, spawn_ball_events, mut paddles): Self::SystemData) {
        for event in spawn_ball_events.read(&mut self.spawn_ball_event_reader.as_mut().unwrap()) {
            let ent = ents.create();

            // If an owning paddle was given, we need to spawn the ball on the paddle. Otherwise use the given spawn position.
            let spawn_pos = match event.owning_paddle_ent {
                Some(paddle_ent) => {
                    let paddle = paddles.get(paddle_ent).unwrap();
                    paddle.held_ball_position
                }
                None => event.position,
            };

            lazy_updater.insert(
                ent,
                TransformComponent::new(spawn_pos, Vector2f::new(1.0, 1.0)),
            );

            let ball_sprite = SpriteRegion {
                x: 64,
                y: 0,
                w: 32,
                h: 32,
            };

            lazy_updater.insert(
                ent,
                SpriteComponent::new(
                    ball_sprite,
                    2,
                    Point2f::new(0.5, 0.5),
                    COLOR_WHITE,
                    2,
                    Transparency::Opaque,
                ),
            );

            lazy_updater.insert(
                ent,
                BallComponent::new(event.linear_velocity, event.owning_paddle_ent),
            );

            lazy_updater.insert(
                ent,
                RigidbodyComponent::new(
                    1.0,
                    event.linear_velocity,
                    BALL_MAX_LINEAR_VELOCITY,
                    BodyStatus::Dynamic,
                ),
            );

            let collision_groups = ncollide2d::pipeline::CollisionGroups::new()
                .with_membership(&[0])
                .with_blacklist(&[0]);
            lazy_updater.insert(
                ent,
                ColliderComponent::new(
                    Ball::new(BALL_COLLIDER_RADIUS * crate::game::WORLD_UNIT_RATIO),
                    Vector2::zeros(),
                    collision_groups,
                    0.0,
                ),
            );

            if let Some(paddle_ent) = event.owning_paddle_ent {
                let mut paddle = paddles
                    .get_mut(paddle_ent)
                    .expect("Failed to spawn ball ent: owning_paddle_ent not found!");
                paddle.held_ball_ent = Some(ent);
            }

            println!("[EntitySpawnSystem] Spawned ball");
        }
    }
}
