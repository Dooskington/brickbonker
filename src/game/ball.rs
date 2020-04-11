use crate::game::{
    brick::BreakableComponent,
    paddle::{PlayerPaddleComponent, PADDLE_HIT_BOX_WIDTH},
    physics::{ColliderComponent, CollisionEvent, PhysicsState, RigidbodyComponent},
    render::SpriteComponent,
    transform::TransformComponent,
    LevelState, Point2f, Vector2d, Vector2f,
};
use gfx::{color::*, sprite::SpriteRegion};
use nalgebra::Vector2;
use ncollide2d::shape::Ball;
use nphysics2d::{math::Velocity, object::BodyStatus};
use shrev::EventChannel;
use specs::prelude::*;

const BALL_MAX_LINEAR_VELOCITY: f64 = 10.0;

#[derive(Clone, Debug)]
pub struct SpawnBallEvent {
    pub position: Vector2d,
    pub linear_velocity: Vector2d,
    pub owning_paddle_ent: Option<Entity>,
}

#[derive(Debug)]
pub struct BallComponent {
    pub last_pos: Point2f,
    velocity: Velocity<f64>,
    is_held: bool,
    did_hit_brick_this_tick: bool,
}

impl BallComponent {
    pub fn new(linear_velocity: Vector2d, is_held: bool) -> Self {
        BallComponent {
            last_pos: Point2f::origin(),
            velocity: Velocity::new(linear_velocity, 0.0),
            is_held,
            did_hit_brick_this_tick: false,
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
        WriteExpect<'a, PhysicsState>,
        Read<'a, LevelState>,
        Read<'a, EventChannel<CollisionEvent>>,
        Write<'a, EventChannel<SpawnBallEvent>>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, BallComponent>,
        WriteStorage<'a, BreakableComponent>,
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
            mut physics,
            level,
            collision_events,
            mut spawn_ball_events,
            mut transforms,
            mut balls,
            mut breakables,
            paddles,
            mut rigidbodies,
        ): Self::SystemData,
    ) {
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
                    vel.x = hit_x_ratio * 4.0;
                    vel.y = vel.y.abs() * -0.975;

                    vel = vel.normalize()
                        * nalgebra::clamp(vel.magnitude(), 0.0, BALL_MAX_LINEAR_VELOCITY);
                    ball.velocity = Velocity::new(vel, 0.0);

                    continue;
                }

                if let Some(normal) = event.normal {
                    let vel = ball.velocity;
                    let normal = -normal.normalize();
                    let dot = vel.linear.dot(&normal);

                    let mut reflected_vel = (vel.linear - (2.0 * dot) * normal) * 1.075;
                    reflected_vel = reflected_vel.normalize()
                        * nalgebra::clamp(reflected_vel.magnitude(), 0.0, BALL_MAX_LINEAR_VELOCITY);
                    ball.velocity = Velocity::new(reflected_vel, vel.angular);

                    crate::game::audio::test_audio();
                } else {
                    println!(
                        "Ball collision had no normal! ball ent = {}, other ent = {}",
                        entity_a.id(),
                        entity_b.id()
                    );
                }
            }
        }

        for (ent, transform, rigidbody, ball) in
            (&ents, &mut transforms, &mut rigidbodies, &mut balls).join()
        {
            if ball.is_held {
                continue;
            }

            // Directly set the ball velocity every tick to keep the physics engine from affecting it
            rigidbody.velocity = ball.velocity;

            // TODO replace this with a sensor collider
            /*
            if transform.position.y > 250.0 {
                ents.delete(ent).expect("Failed to delete ball ent!");

                use rand::Rng;
                let mut rand = rand::thread_rng();
                let position = Vector2d::new(rand.gen_range(64.0, 200.0), 64.0);
                let linear_velocity =
                    Vector2d::new(rand.gen_range(-6.0, 6.0), rand.gen_range(-3.0, 5.0));

                spawn_ball_events.single_write(SpawnBallEvent {
                    position,
                    linear_velocity,
                    //owning_paddle_ent: level.player_paddle_ent,
                    owning_paddle_ent: None,
                });

                continue;
            }
            */
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

            lazy_updater.insert(
                ent,
                TransformComponent {
                    position: event.position,
                    last_position: event.position,
                    origin: Point2f::new(16.0, 16.0),
                    scale: Vector2f::new(1.0, 1.0),
                },
            );

            lazy_updater.insert(
                ent,
                SpriteComponent {
                    color: COLOR_WHITE,
                    spritesheet_tex_id: 2,
                    region: SpriteRegion {
                        x: 64,
                        y: 0,
                        w: 32,
                        h: 32,
                    },
                    layer: 0,
                },
            );

            lazy_updater.insert(
                ent,
                BallComponent::new(event.linear_velocity, event.owning_paddle_ent.is_some()),
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
                ColliderComponent::new(Ball::new(0.125), Vector2::zeros(), collision_groups, 0.0),
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
