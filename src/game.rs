use crate::physics::*;
use gfx::{color::*, input::*, renderer::*, sprite::*, Point2f, Vector2f};
use nalgebra::Vector2;
use ncollide2d::shape::{Ball, Cuboid, Capsule, ShapeHandle};
use nphysics2d::object::Body;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::math::Velocity;
use nphysics2d::object::{
    BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodySet, DefaultColliderSet, Ground,
    RigidBodyDesc,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
use shrev::EventChannel;
use specs::prelude::*;
use std::collections::HashMap;

pub const PIXELS_PER_WORLD_UNIT: u32 = 32;
pub const WORLD_UNIT_RATIO: f64 = (1.0 / PIXELS_PER_WORLD_UNIT as f64);

const PADDLE_SPRITE_WIDTH: u32 = 64;
const PADDLE_SPRITE_HEIGHT: u32 = 32;
const PADDLE_BB_X: f32 = 3.0;
const PADDLE_BB_Y: f32 = 10.0;
const PADDLE_BB_WIDTH: f32 = 58.0;
const PADDLE_BB_HEIGHT: f32 = 10.0;
const PADDLE_SCALE_X: f32 = 1.0;
const PADDLE_SCALE_Y: f32 = 1.0;

const DEFAULT_BALL_FORCE: f32 = 2.0;
const BALL_SPRITE_WIDTH: u32 = 32;
const BALL_SPRITE_HEIGHT: u32 = 32;
const BALL_SCALE_X: f32 = 1.0;
const BALL_SCALE_Y: f32 = 1.0;
const BALL_BB_RADIUS: f32 = 5.0;
const BALL_MAX_AXIS_VELOCITY: f32 = 6.5;

const DEFAULT_BRICK_HP: i32 = 1;
const BRICK_SPRITE_WIDTH: u32 = 32;
const BRICK_SPRITE_HEIGHT: u32 = 16;
const BRICK_SCALE_X: f32 = 1.0;
const BRICK_SCALE_Y: f32 = 1.0;

const LEVEL_BRICKS_WIDTH: usize = 18;
const LEVEL_BRICKS_HEIGHT: usize = 10;

pub struct GameState<'a, 'b> {
    pub world: World,
    pub tick_dispatcher: Dispatcher<'a, 'b>,
    pub physics_dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> GameState<'a, 'b> {
    pub fn new(width: u32, height: u32) -> GameState<'a, 'b> {
        let mut world = World::new();

        let mut tick_dispatcher = DispatcherBuilder::new()
            .with(BallSystem::default(), "ball_physics", &[])
            .with(PlayerPaddleSystem, "player_paddle", &[])
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

        let solid_collision_groups = ncollide2d::pipeline::CollisionGroups::new().with_membership(&[1]);

        // Spawn paddle ent
        /*
        let paddle_ent = world
            .create_entity()
            .with(TransformComponent {
                pos_x: width as f64 / 2.0,
                pos_y: height as f64 - 6.0,
                last_pos_x: 0.0,
                last_pos_y: 0.0,
                origin: Point2f::new(32.0, 16.0),
                scale: Vector2f::new(PADDLE_SCALE_X, PADDLE_SCALE_Y),
            })
            .with(ColliderComponent::new(
                Cuboid::new(Vector2::new(29.0 * WORLD_UNIT_RATIO, 4.0 * WORLD_UNIT_RATIO)),
                Vector2::zeros(),
                solid_collision_groups,
            ))
            .with(PlayerPaddleComponent::default())
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
        */

        // Spawn brick ents
        /*
        for x in 0..LEVEL_BRICKS_WIDTH {
            for y in 0..LEVEL_BRICKS_HEIGHT {
                world
                    .create_entity()
                    .with(TransformComponent {
                        pos_x: 32.0 + (x as f32 * (BRICK_SPRITE_WIDTH as f32 * BRICK_SCALE_X)),
                        pos_y: 32.0 + (y as f32 * (BRICK_SPRITE_HEIGHT as f32 * BRICK_SCALE_Y)),
                        scale: Vector2f::new(BRICK_SCALE_X, BRICK_SCALE_Y),
                        origin: Point2f::new(0.0, 16.0),
                    })
                    .with(BoundingBoxComponent {
                        x: 0.0,
                        y: 0.0,
                        w: 32.0,
                        h: 16.0,
                        bb: None,
                    })
                    .with(BreakableComponent {
                        hp: DEFAULT_BRICK_HP,
                    })
                    .with(SpriteComponent {
                        color: COLOR_WHITE,
                        spritesheet_tex_id: 2,
                        region: SpriteRegion {
                            x: 96,
                            y: 0,
                            w: BRICK_SPRITE_WIDTH,
                            h: BRICK_SPRITE_HEIGHT,
                        },
                    })
                    .build();
            }
        }
        */

        // test bricks
        world
            .create_entity()
            .with(TransformComponent {
                pos_x: 256.0,
                pos_y: 32.0,
                scale: Vector2f::new(BRICK_SCALE_X, BRICK_SCALE_Y),
                origin: Point2f::new(16.0, 8.0),
                ..Default::default()
            })
            .with(ColliderComponent::new(Cuboid::new(Vector2::new(0.5, 0.25)), Vector2::zeros(), solid_collision_groups, 1.0))
            .with(BreakableComponent {
                hp: DEFAULT_BRICK_HP,
            })
            .with(SpriteComponent {
                color: COLOR_WHITE,
                spritesheet_tex_id: 2,
                region: SpriteRegion {
                    x: 96,
                    y: 0,
                    w: BRICK_SPRITE_WIDTH,
                    h: BRICK_SPRITE_HEIGHT,
                },
                layer: 0,
            })
            .build();

        world
            .create_entity()
            .with(TransformComponent {
                pos_x: 64.0,
                pos_y: 32.0,
                scale: Vector2f::new(BRICK_SCALE_X, BRICK_SCALE_Y),
                origin: Point2f::new(16.0, 8.0),
                ..Default::default()
            })
            .with(ColliderComponent::new(Cuboid::new(Vector2::new(0.5, 0.25)), Vector2::zeros(), solid_collision_groups, 1.0))
            .with(BreakableComponent {
                hp: DEFAULT_BRICK_HP,
            })
            .with(SpriteComponent {
                color: COLOR_WHITE,
                spritesheet_tex_id: 2,
                region: SpriteRegion {
                    x: 96,
                    y: 0,
                    w: BRICK_SPRITE_WIDTH,
                    h: BRICK_SPRITE_HEIGHT,
                },
                layer: 0,
            })
            .build();


        world
            .create_entity()
            .with(TransformComponent {
                pos_x: width as f64 / 2.0,
                pos_y: height as f64 - 50.0,
                scale: Vector2f::new(2.0, 2.0),
                origin: Point2f::new(16.0, 8.0),
                ..Default::default()
            })
            .with(ColliderComponent::new(Cuboid::new(Vector2::new(1.0, 0.5)), Vector2::zeros(), solid_collision_groups, 1.0))
            .with(BreakableComponent {
                hp: DEFAULT_BRICK_HP,
            })
            .with(SpriteComponent {
                color: COLOR_WHITE,
                spritesheet_tex_id: 2,
                region: SpriteRegion {
                    x: 96,
                    y: 0,
                    w: BRICK_SPRITE_WIDTH,
                    h: BRICK_SPRITE_HEIGHT,
                },
                layer: 0,
            })
            .build();

        // Spawn the initial ball
        world
            .write_resource::<EventChannel<SpawnBallEvent>>()
            .single_write(SpawnBallEvent {
                pos_x: width as f64 / 2.0,
                pos_y: height as f64 / 2.0,
                vel_x: 3.5,
                vel_y: -4.5,
                //owning_paddle_ent: Some(paddle_ent),
                owning_paddle_ent: None,
            });

        world
            .write_resource::<EventChannel<SpawnBallEvent>>()
            .single_write(SpawnBallEvent {
                pos_x: 40.0,
                pos_y: height as f64 - 50.0,
                vel_x: 2.0,
                vel_y: 4.0,
                //owning_paddle_ent: Some(paddle_ent),
                owning_paddle_ent: None,
            });

        world
            .write_resource::<EventChannel<SpawnBallEvent>>()
            .single_write(SpawnBallEvent {
                pos_x: width as f64 / 2.0,
                pos_y: height as f64 / 2.0,
                vel_x: -4.7,
                vel_y: 4.1,
                //owning_paddle_ent: Some(paddle_ent),
                owning_paddle_ent: None,
            });

        world
            .write_resource::<EventChannel<SpawnBallEvent>>()
            .single_write(SpawnBallEvent {
                pos_x: width as f64 / 2.0,
                pos_y: height as f64 / 2.0,
                vel_x: -4.3,
                vel_y: 4.1,
                //owning_paddle_ent: Some(paddle_ent),
                owning_paddle_ent: None,
            });

        world
            .write_resource::<EventChannel<SpawnBallEvent>>()
            .single_write(SpawnBallEvent {
                pos_x: width as f64 / 2.0,
                pos_y: height as f64 / 2.0,
                vel_x: -4.7,
                vel_y: -1.1,
                //owning_paddle_ent: Some(paddle_ent),
                owning_paddle_ent: None,
            });

        world
            .write_resource::<EventChannel<SpawnBallEvent>>()
            .single_write(SpawnBallEvent {
                pos_x: width as f64 / 2.0,
                pos_y: height as f64 / 2.0,
                vel_x: 2.7,
                vel_y: -5.1,
                //owning_paddle_ent: Some(paddle_ent),
                owning_paddle_ent: None,
            });

        /*
        world
            .write_resource::<EventChannel<SpawnBallEvent>>()
            .single_write(SpawnBallEvent {
                pos_x: width as f64 / 2.0,
                pos_y: height as f64 / 2.0,
                vel_x: 4.0,
                vel_y: 4.0,
                //owning_paddle_ent: Some(paddle_ent),
                owning_paddle_ent: None,
            });
            */

        // Bottom collider
        world
            .create_entity()
            .with(TransformComponent {
                pos_x: 0.0,
                pos_y: height as f64 + 8.0,
                ..Default::default()
            })
            .with(ColliderComponent::new(
                Cuboid::new(Vector2::new(50.0, 10.0 * WORLD_UNIT_RATIO)),
                Vector2::zeros(),
                solid_collision_groups,
                1.0,
            ))
            .build();

        // Left collider
        world
            .create_entity()
            .with(TransformComponent {
                pos_x: -20.0,
                pos_y: 0.0,
                ..Default::default()
            })
            .with(ColliderComponent::new(
                Cuboid::new(Vector2::new(20.0 * WORLD_UNIT_RATIO, 50.0)),
                Vector2::zeros(),
                solid_collision_groups,
                1.0,
            ))
            .build();

        // Top collider
        world
            .create_entity()
            .with(TransformComponent {
                pos_x: 0.0,
                pos_y: -20.0,
                ..Default::default()
            })
            .with(ColliderComponent::new(
                Cuboid::new(Vector2::new(50.0, 20.0 * WORLD_UNIT_RATIO)),
                Vector2::zeros(),
                solid_collision_groups,
                1.0,
            ))
            .build();

        // Right collider
        world
            .create_entity()
            .with(TransformComponent {
                pos_x: width as f64 + 20.0,
                pos_y: 0.0,
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
        world.insert(RenderCommander::new());
        world.insert(LevelState {
            level: 1,
            player_paddle_ent: None,
            //player_paddle_ent: Some(paddle_ent),
        });
        world.insert(PhysicsState::new());

        GameState {
            world,
            tick_dispatcher,
            physics_dispatcher,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnBallEvent {
    pub pos_x: f64,
    pub pos_y: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub owning_paddle_ent: Option<Entity>,
}

#[derive(Default)]
pub struct LevelState {
    pub level: i32,
    pub player_paddle_ent: Option<Entity>,
}

#[derive(Default)]
pub struct RenderCommander {
    commands: Vec<gfx::renderer::RenderCommand>,
    bound_transparency: Transparency,
    bound_texture_id: TextureId,
    bound_layer: u8,
    bound_color: Color,
}

impl RenderCommander {
    pub fn new() -> Self {
        RenderCommander {
            ..Default::default()
        }
    }

    pub fn bind_transparency(&mut self, val: Transparency) {
        self.bound_transparency = val;
    }

    pub fn bind_texture(&mut self, val: TextureId) {
        self.bound_texture_id = val;
    }

    pub fn bind_layer(&mut self, val: u8) {
        self.bound_layer = val;
    }

    pub fn bind_color(&mut self, val: Color) {
        self.bound_color = val;
    }

    pub fn sprite(
        &mut self,
        x: f32,
        y: f32,
        origin: Point2f,
        scale: Vector2f,
        region: SpriteRegion,
    ) {
        self.commands.push(gfx::renderer::RenderCommand {
            transparency: self.bound_transparency,
            shader_program_id: 1,
            tex_id: self.bound_texture_id,
            layer: self.bound_layer,
            data: Renderable::Sprite {
                x,
                y,
                origin,
                scale,
                color: self.bound_color,
                region,
            },
        });
    }

    pub fn clear_commands(&mut self) {
        self.bound_transparency = Transparency::default();
        self.bound_texture_id = 0;
        self.bound_layer = 0;
        self.bound_color = Color::default();
        self.commands.clear();
    }

    pub fn commands(&mut self) -> Vec<gfx::renderer::RenderCommand> {
        self.commands.clone()
    }
}

#[derive(Debug)]
pub struct TransformComponent {
    pub pos_x: f64,
    pub pos_y: f64,
    pub last_pos_x: f64,
    pub last_pos_y: f64,
    pub origin: Point2f,
    pub scale: Vector2f,
}

impl Component for TransformComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            pos_x: 0.0,
            pos_y: 0.0,
            last_pos_x: 0.0,
            last_pos_y: 0.0,
            origin: Point2f::origin(),
            scale: Vector2f::new(1.0, 1.0),
        }
    }
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

#[derive(Debug)]
pub struct SpriteComponent {
    pub color: Color,
    pub region: SpriteRegion,
    pub spritesheet_tex_id: TextureId,
    pub layer: u8,
}

impl Component for SpriteComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct PlayerPaddleComponent {
    pub vel_x: f32,
    pub held_ball_ent: Option<Entity>,
    pub held_ball_pos_x: f32,
    pub held_ball_pos_y: f32,
}

impl Component for PlayerPaddleComponent {
    type Storage = VecStorage<Self>;
}

pub struct BreakableComponent {
    pub hp: i32,
}

impl Component for BreakableComponent {
    type Storage = VecStorage<Self>;
}

struct PlayerPaddleSystem;

impl<'a> System<'a> for PlayerPaddleSystem {
    type SystemData = (
        Read<'a, InputState>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, PlayerPaddleComponent>,
        WriteStorage<'a, BallComponent>,
    );

    fn run(&mut self, (input, mut transforms, mut paddles, mut balls): Self::SystemData) {
        for (transform, paddle) in (&mut transforms, &mut paddles).join() {
            let is_moving_left =
                input.is_key_held(VirtualKeyCode::A) || input.is_key_held(VirtualKeyCode::Left);
            let is_moving_right =
                input.is_key_held(VirtualKeyCode::D) || input.is_key_held(VirtualKeyCode::Right);

            let speed = 400.0 * 0.016;
            paddle.vel_x = 0.0;

            if is_moving_left {
                paddle.vel_x -= speed;
            }

            if is_moving_right {
                paddle.vel_x += speed;
            }

            transform.pos_x += paddle.vel_x as f64;

            if transform.pos_x < (PADDLE_SPRITE_WIDTH as f64 / 2.0) {
                transform.pos_x = PADDLE_SPRITE_WIDTH as f64 / 2.0;
            } else if transform.pos_x > (640.0 - (PADDLE_SPRITE_WIDTH as f64 / 2.0)) {
                transform.pos_x = 640.0 - (PADDLE_SPRITE_WIDTH as f64 / 2.0);
            }

            /*
            paddle.held_ball_pos_x = transform.pos_x as f64;
            paddle.held_ball_pos_y =
                transform.pos_y - (PADDLE_BB_HEIGHT as f64 * PADDLE_SCALE_Y) - BALL_BB_RADIUS;
            */
        }

        // Handle paddles that are holding a ball
        /*
        for mut paddle in (&mut paddles).join() {
            if let Some(ball_ent) = paddle.held_ball_ent {
                let ball_transform = transforms.get_mut(ball_ent).expect(
                    "Failed to set held_ball_ent position! Entity had no TransformComponent!",
                );
                ball_transform.pos_x = paddle.held_ball_pos_x;
                ball_transform.pos_y = paddle.held_ball_pos_y;

                if input.is_key_pressed(VirtualKeyCode::Space) {
                    paddle.held_ball_ent = None;

                    let ball = balls.get_mut(ball_ent).expect(
                        "Failed to set held_ball_ent position! Entity had no BallComponent!",
                    );

                    ball.is_held = false;
                    ball.vel_x = paddle.vel_x * 0.5;
                    ball.vel_y = -DEFAULT_BALL_FORCE;
                }
            }
        }
        */
    }
}

#[derive(Default)]
struct SpriteRenderSystem;

impl<'a> System<'a> for SpriteRenderSystem {
    type SystemData = (
        ReadExpect<'a, PhysicsState>,
        Write<'a, RenderCommander>,
        ReadStorage<'a, TransformComponent>,
        ReadStorage<'a, SpriteComponent>,
        ReadStorage<'a, RigidbodyComponent>,
    );

    fn run(&mut self, (physics, mut render, transforms, sprites, rigidbodies): Self::SystemData) {
        for (transform, sprite, rigidbody) in (&transforms, &sprites, (&rigidbodies).maybe()).join()
        {
            let (x, y) = if let Some(_) = rigidbody {
                let x = (transform.pos_x * physics.lerp)
                    + (transform.last_pos_x * (1.0 - physics.lerp));
                let y = (transform.pos_y * physics.lerp)
                    + (transform.last_pos_y * (1.0 - physics.lerp));
                (x, y)
            } else {
                (transform.pos_x, transform.pos_y)
            };

            render.bind_texture(sprite.spritesheet_tex_id);
            render.bind_color(sprite.color);
            render.bind_layer(sprite.layer);
            render.sprite(
                x as f32,
                y as f32,
                transform.origin,
                transform.scale,
                sprite.region,
            );
        }
    }
}

#[derive(Default)]
struct BallSystem {
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

        for (ent, transform, rigidbody, ball) in (&ents, &mut transforms, &mut rigidbodies, &mut balls).join() {
            if ball.is_held {
                continue;
            }

            // Directly set the ball velocity every tick to keep the physics engine from affecting it
            rigidbody.velocity = rigidbody.continuous_velocity;

            if transform.pos_y > 250.0 {
                use rand::Rng;
                let mut rand =  rand::thread_rng();
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
struct SpawnBallSystem {
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
                    scale: Vector2f::new(BALL_SCALE_X, BALL_SCALE_Y),
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
                        w: BALL_SPRITE_WIDTH,
                        h: BALL_SPRITE_HEIGHT,
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
                RigidbodyComponent::new(1.0, Vector2::zeros(), Vector2::new(event.vel_x, event.vel_y)),
            );

            let collision_groups = ncollide2d::pipeline::CollisionGroups::new().with_membership(&[0]).with_blacklist(&[0]);
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
