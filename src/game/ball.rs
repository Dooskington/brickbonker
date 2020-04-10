use crate::game::{
    brick::BreakableComponent,
    paddle::PlayerPaddleComponent,
    physics::{ColliderComponent, CollisionEvent, PhysicsState, RigidbodyComponent},
    render::SpriteComponent,
    transform::TransformComponent,
    LevelState, Point2f, Vector2f,
};
use gfx::{color::*, sprite::SpriteRegion};
use nalgebra::Vector2;
use ncollide2d::shape::Ball;
use nphysics2d::math::Velocity;
use shrev::EventChannel;
use specs::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnBallEvent {
    pub pos_x: f64,
    pub pos_y: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub owning_paddle_ent: Option<Entity>,
}

#[derive(Debug)]
pub struct BallComponent {
    pub last_pos: Point2f,
    vel_x: f64,
    vel_y: f64,
    is_held: bool,
    did_hit_brick_this_tick: bool,
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
        // Keep track of any balls that bounce due to collision during this tick.
        // We will use this to make sure we don't react to more than one collision per ball per tick
        // (This fixes the scenario where a ball hits the corner of the world perfectly causing two reflections on one tick, getting stuck)
        let mut bounced_balls_set: BitSet = BitSet::new();
        for event in collision_events.read(&mut self.collision_event_reader.as_mut().unwrap()) {
            // Get the entities involved in the event, ignoring it entirely if either of them are not an entity
            let (entity_a, entity_b) = {
                if event.entity_a.is_none() || event.entity_b.is_none() {
                    continue;
                }

                (event.entity_a.unwrap(), event.entity_b.unwrap())
            };

            // Figure out which of the entities was a ball (if any)

            if let Some(rigidbody) = rigidbodies.get_mut(entity_a) {
                // If the ball hit a paddle, propel it upwards
                if let Some(_) = paddles.get(entity_b) {
                    let x_vel_multiplier = 0.85;
                    let y_vel_multiplier = -0.85;
                    let mut vel = rigidbody.continuous_velocity.linear;
                    vel.x *= x_vel_multiplier;
                    vel.y *= y_vel_multiplier;

                    vel = vel.normalize() * nalgebra::clamp(vel.magnitude(), 0.1, 2.0);
                    //rigidbody.velocity = Velocity::linear(vel.x, vel.y);
                    println!("Hit paddle!");
                    continue;
                }

                if let Some(normal) = event.normal {
                    if bounced_balls_set.contains(entity_a.id()) {
                        println!("entity_a already bounced this tick");
                        //continue;
                    }

                    let vel = rigidbody.continuous_velocity;
                    let normal = -normal.normalize();
                    let dot = vel.linear.dot(&normal);

                    /*
                    let force_multiplier = 1.025;
                    let mut reflected_vel = (vel.linear - (2.0 * dot) * normal) * force_multiplier;

                    reflected_vel = reflected_vel.normalize() * nalgebra::clamp(reflected_vel.magnitude(), 0.1, 2.0);
                    */
                    //rigidbody.velocity = Velocity::new(reflected_vel, vel.angular);
                    //rigidbody.velocity = Velocity::linear(0.0, 0.0);

                    let reflected_vel = vel.linear - (2.0 * dot) * normal;
                    rigidbody.continuous_velocity = Velocity::new(reflected_vel, vel.angular);

                    crate::game::audio::test_audio();

                    bounced_balls_set.add(entity_a.id());

                //println!("a: {:?} vel: {:?}  normal: {:?} reflection: {:?}", vel.linear, entity_a.id(), normal, reflected_vel);
                } else {
                    println!("ERROR! entity_a collision had no normal!");
                }
            } else if let Some(rigidbody) = rigidbodies.get_mut(entity_b) {
                // If the ball hit a paddle, propel it upwards
                if let Some(_) = paddles.get(entity_a) {
                    let x_vel_multiplier = 0.85;
                    let y_vel_multiplier = -0.85;
                    let mut vel = rigidbody.continuous_velocity.linear;
                    vel.x *= x_vel_multiplier;
                    vel.y *= y_vel_multiplier;

                    vel = vel.normalize() * nalgebra::clamp(vel.magnitude(), 0.1, 2.0);
                    //rigidbody.velocity = Velocity::linear(vel.x, vel.y);
                    println!("Hit paddle!");
                    continue;
                }

                if let Some(normal) = event.normal {
                    if bounced_balls_set.contains(entity_b.id()) {
                        println!("entity_b already bounced this tick");
                        //continue;
                    }

                    let vel = rigidbody.continuous_velocity;
                    let normal = normal.normalize();
                    let dot = vel.linear.dot(&normal);

                    /*
                    let force_multiplier = 1.025;
                    let mut reflected_vel = (vel.linear - (2.0 * dot) * normal) * force_multiplier;

                    reflected_vel = reflected_vel.normalize() * nalgebra::clamp(reflected_vel.magnitude(), 0.1, 2.0);
                    */
                    //rigidbody.velocity = Velocity::new(reflected_vel, vel.angular);
                    //rigidbody.velocity = Velocity::linear(0.0, 0.0);

                    let reflected_vel = vel.linear - (2.0 * dot) * normal;
                    rigidbody.continuous_velocity = Velocity::new(reflected_vel, vel.angular);

                    crate::game::audio::test_audio();

                    bounced_balls_set.add(entity_b.id());

                //println!("b: {} vel: {:?} normal: {:?} reflection: {:?}", vel.linear, entity_b.id(), normal, reflected_vel);
                } else {
                    println!("ERROR! entity_b collision had no normal!");
                }
            } else {
                // TODO

                println!("ERROR! entity_a AND entity_b were not rigidbodies");
            }
        }

        for (ent, transform, rigidbody, ball) in
            (&ents, &mut transforms, &mut rigidbodies, &mut balls).join()
        {
            if ball.is_held {
                continue;
            }

            // Directly set the ball velocity every tick to keep the physics engine from affecting it
            rigidbody.velocity = rigidbody.continuous_velocity;

            if transform.pos_y > 250.0 {
                use rand::Rng;
                let mut rand = rand::thread_rng();
                let vel_x = rand.gen_range(-6.0, 6.0);
                let vel_y = rand.gen_range(-3.0, 5.0);

                ents.delete(ent).expect("Failed to delete ball ent!");
                spawn_ball_events.single_write(SpawnBallEvent {
                    pos_x: rand.gen_range(64.0, 128.0),
                    pos_y: 64.0,
                    vel_x,
                    vel_y,
                    //owning_paddle_ent: level.player_paddle_ent,
                    owning_paddle_ent: None,
                });

                continue;
            }

            /*
            // Handle the collision (if any)
            match collision {
                // Paddle collision
                BallCollision::Paddle { x_hit_percentage } => {
                    let paddle_hit_force: f32 = 2.0;
                    let temp_velocity = Vector2f::new(ball.vel_x, ball.vel_y);
                    ball.vel_x = DEFAULT_BALL_FORCE * x_hit_percentage * paddle_hit_force;

                    // Always propel the ball upwards - this fixes issues when the ball hits the side or underneath the paddle.
                    ball.vel_y = -1.0 * (ball.vel_y.abs() * 0.9);

                    let normalized = Vector2::<f32>::new(ball.vel_x, ball.vel_y).normalize();
                    let new_velocity = normalized * temp_velocity.magnitude();

                    ball.vel_x = new_velocity.x;
                    ball.vel_y = new_velocity.y;
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
                    pos_x: event.pos_x,
                    pos_y: event.pos_y,
                    last_pos_x: event.pos_x,
                    last_pos_y: event.pos_y,
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
                BallComponent {
                    last_pos: Point2f::new(0.0, 0.0),
                    vel_x: event.vel_x,
                    vel_y: event.vel_y,
                    is_held: event.owning_paddle_ent.is_some(),
                    did_hit_brick_this_tick: false,
                },
            );

            lazy_updater.insert(
                ent,
                RigidbodyComponent::new(
                    1.0,
                    Vector2::zeros(),
                    Vector2::new(event.vel_x, event.vel_y),
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
