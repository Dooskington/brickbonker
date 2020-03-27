use gfx::{color::*, input::*, renderer::*, sprite::*, Point2f, Vector2f};
use nalgebra::Vector2;
use shrev::EventChannel;
use specs::prelude::*;
use std::collections::HashMap;

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
}

impl<'a, 'b> GameState<'a, 'b> {
    pub fn new() -> GameState<'a, 'b> {
        let mut world = World::new();

        // Components
        world.register::<TransformComponent>();
        world.register::<BallComponent>();
        world.register::<SpriteComponent>();
        world.register::<PlayerPaddleComponent>();
        world.register::<BoundingBoxComponent>();
        world.register::<BreakableComponent>();

        // Create paddle ent
        let paddle_ent = world
            .create_entity()
            .with(TransformComponent {
                pos_x: 64.0,
                pos_y: 470.0,
                origin: Point2f::new(32.0, 20.0),
                scale: Vector2f::new(PADDLE_SCALE_X, PADDLE_SCALE_Y),
            })
            .with(BoundingBoxComponent {
                x: PADDLE_BB_X,
                y: PADDLE_BB_Y,
                w: PADDLE_BB_WIDTH,
                h: PADDLE_BB_HEIGHT,
                bb: None,
            })
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
            })
            .build();

        // Create brick ents
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

        // Resources
        world.insert(RenderCommander::new());
        world.insert(WorldBoundingBoxState::default());
        world.insert(LevelState {
            level: 1,
            player_paddle_ent: Some(paddle_ent),
        });

        let mut tick_dispatcher = DispatcherBuilder::new()
            .with(BallPhysicsSystem, "ball_physics", &[])
            .with(PlayerPaddleSystem, "player_paddle", &[])
            .with(
                BoundingBoxSystem,
                "bounding_box",
                &["player_paddle", "ball_physics"],
            )
            .with_thread_local(SpawnBallSystem::default())
            .with_thread_local(SpriteRenderSystem {})
            .build();

        tick_dispatcher.setup(&mut world);

        // Spawn the initial ball
        world
            .write_resource::<EventChannel<SpawnBallEvent>>()
            .single_write(SpawnBallEvent {
                pos_x: 0.0,
                pos_y: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                owning_paddle_ent: Some(paddle_ent),
            });

        GameState {
            world,
            tick_dispatcher,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnBallEvent {
    pub pos_x: f32,
    pub pos_y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub owning_paddle_ent: Option<Entity>,
}

/// Ball collision types and associated data
#[derive(Clone, Copy, PartialEq)]
pub enum BallCollision {
    Brick {
        hit_normal: Vector2f,
        collision_dir: AABBCollisionDirection,
        hit_ent: Entity,
    },
    Paddle {
        x_hit_percentage: f32,
    },
    None,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AABBCollisionDirection {
    Up,
    Down,
    Left,
    Right,
}

impl AABBCollisionDirection {
    /// Calculate the direction that the collision is coming from, using the collision vector
    pub fn from_vector(vector: Vector2<f32>) -> AABBCollisionDirection {
        let dirs: Vec<(Vector2<f32>, AABBCollisionDirection)> = vec![
            (Vector2::new(0.0, -1.0), AABBCollisionDirection::Up),
            (Vector2::new(0.0, 1.0), AABBCollisionDirection::Down),
            (Vector2::new(1.0, 0.0), AABBCollisionDirection::Left),
            (Vector2::new(-1.0, 0.0), AABBCollisionDirection::Right),
        ];

        let mut max: f32 = 0.0;
        let mut best_match: AABBCollisionDirection = AABBCollisionDirection::Up;
        for (v, dir) in dirs {
            let dot_product: f32 = vector.dot(&v);
            if dot_product > max {
                max = dot_product;
                best_match = dir;
            }
        }

        return best_match;
    }
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub left_x: f32,
    pub right_x: f32,
    pub top_y: f32,
    pub bottom_y: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub half_w: f32,
    pub half_h: f32,
}

#[derive(Default)]
pub struct WorldBoundingBoxState {
    pub boxes: HashMap<Entity, BoundingBox>,
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
    pub pos_x: f32,
    pub pos_y: f32,
    pub origin: Point2f,
    pub scale: Vector2f,
}

impl Component for TransformComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct BallComponent {
    pub last_pos: Point2f,
    vel_x: f32,
    vel_y: f32,
    is_held: bool,
    did_hit_brick_this_tick: bool,
}

impl Component for BallComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct BoundingBoxComponent {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub bb: Option<BoundingBox>,
}

impl Component for BoundingBoxComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct SpriteComponent {
    pub color: Color,
    pub region: SpriteRegion,
    pub spritesheet_tex_id: TextureId,
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

            let speed = 10.0;
            paddle.vel_x = 0.0;

            if is_moving_left {
                paddle.vel_x -= speed;
            }

            if is_moving_right {
                paddle.vel_x += speed;
            }

            transform.pos_x += paddle.vel_x;

            if transform.pos_x < (PADDLE_SPRITE_WIDTH as f32 / 2.0) {
                transform.pos_x = PADDLE_SPRITE_WIDTH as f32 / 2.0;
            } else if transform.pos_x > (640.0 - (PADDLE_SPRITE_WIDTH as f32 / 2.0)) {
                transform.pos_x = 640.0 - (PADDLE_SPRITE_WIDTH as f32 / 2.0);
            }

            paddle.held_ball_pos_x = transform.pos_x;
            paddle.held_ball_pos_y =
                transform.pos_y - (PADDLE_BB_HEIGHT as f32 * PADDLE_SCALE_Y) - BALL_BB_RADIUS;
        }

        // Handle paddles that are holding a ball
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
    }
}

struct SpriteRenderSystem;

impl<'a> System<'a> for SpriteRenderSystem {
    type SystemData = (
        Write<'a, RenderCommander>,
        ReadStorage<'a, TransformComponent>,
        ReadStorage<'a, SpriteComponent>,
    );

    fn run(&mut self, (mut render, transforms, sprites): Self::SystemData) {
        for (transform, sprite) in (&transforms, &sprites).join() {
            render.bind_texture(sprite.spritesheet_tex_id);
            render.bind_color(sprite.color);
            render.sprite(
                transform.pos_x,
                transform.pos_y,
                transform.origin,
                transform.scale,
                sprite.region,
            );
        }
    }
}

struct BallPhysicsSystem;

impl<'a> System<'a> for BallPhysicsSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, WorldBoundingBoxState>,
        Read<'a, LevelState>,
        Write<'a, EventChannel<SpawnBallEvent>>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, BallComponent>,
        WriteStorage<'a, BreakableComponent>,
        ReadStorage<'a, PlayerPaddleComponent>,
    );

    fn run(
        &mut self,
        (
            ents,
            world_bounding_boxes,
            level,
            mut spawn_ball_events,
            mut transforms,
            mut balls,
            mut breakables,
            paddles,
        ): Self::SystemData,
    ) {
        for (ent, transform, ball) in (&ents, &mut transforms, &mut balls).join() {
            if ball.is_held {
                continue;
            }

            let mut correction: Option<Vector2f> = None;

            // Check for wall collisions
            if transform.pos_x < 0.0 {
                transform.pos_x = 0.0;
                ball.vel_x = ball.vel_x.abs() * 1.1;
            } else if transform.pos_x > 640.0 {
                transform.pos_x = 640.0;
                ball.vel_x = ball.vel_x.abs() * -1.1;
            }
            // Check for ceiling colision
            else if transform.pos_y < 0.0 {
                transform.pos_y = 0.0;
                ball.vel_y = ball.vel_y.abs() * 1.1;
            }
            // Check for out of bounds (below paddle)
            else if transform.pos_y > 480.0 {
                transform.pos_y = 480.0;
                ball.vel_x = 0.0;
                ball.vel_y = 0.0;

                ents.delete(ent).expect("Failed to delete ball ent!");
                spawn_ball_events.single_write(SpawnBallEvent {
                    pos_x: 0.0,
                    pos_y: 0.0,
                    vel_x: 0.0,
                    vel_y: 0.0,
                    owning_paddle_ent: level.player_paddle_ent,
                });
            }
            // Check for brick collisions
            else {
                let ball_center = Point2f::new(transform.pos_x, transform.pos_y);

                let mut collision = check_ball_brick_collision(
                    ent.id(),
                    ball_center,
                    world_bounding_boxes.boxes.iter(),
                    &paddles,
                );

                // If there was a collision, also check for collision at our interpolated position (halfway between last and current position).
                // If we also get a collision here, it will be the more accurate result.
                if let BallCollision::Brick { .. } = collision {
                    let movement_dist = nalgebra::distance(&ball_center, &ball.last_pos);
                    let dir = (ball_center - ball.last_pos).normalize();
                    let interpolated = ball_center - (dir * (movement_dist / 2.0));

                    println!("  - Checking interpolated collision at {}", interpolated);
                    let interpolated_pos_collision = check_ball_brick_collision(
                        ent.id(),
                        interpolated,
                        world_bounding_boxes.boxes.iter(),
                        &paddles,
                    );

                    if interpolated_pos_collision != BallCollision::None {
                        println!("      - had collision AGAIN");
                        collision = interpolated_pos_collision;
                    }

                    /*
                    println!("  - Checking last_pos collision at {}", ball.last_pos);
                    let last_pos_collision = check_ball_brick_collision(
                        ent.id(),
                        ball.last_pos,
                        world_bounding_boxes.boxes.iter(),
                        &paddles,
                    );

                    if last_pos_collision != BallCollision::None {
                        println!("      - had collision AT LAST POS!");
                        collision = last_pos_collision;
                    }
                    */
                }

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
                    // Brick collision
                    BallCollision::Brick {
                        hit_normal,
                        collision_dir,
                        hit_ent,
                    } => {
                        correction = Some(hit_normal * BALL_BB_RADIUS);
                        //println!("{}, {} CORRECTION: {}, before normalize: {}, dist: {}, closest: {}, {}, ball: {}, {}", transform.pos_x, transform.pos_y, correction, Vector2f::new(diff_x, diff_y), dist, closest_x, closest_y, ball_center_x, ball_center_y);
                        //transform.pos_x += correction.x;
                        //transform.pos_y += correction.y;

                        if (collision_dir == AABBCollisionDirection::Up)
                            || (collision_dir == AABBCollisionDirection::Down)
                        {
                            ball.vel_y *= -1.0;
                        } else if (collision_dir == AABBCollisionDirection::Left)
                            || (collision_dir == AABBCollisionDirection::Right)
                        {
                            ball.vel_x *= -1.0;
                        }

                        //let current_magnitude = Vector2f::new(ball.vel_x, ball.vel_y).magnitude();
                        //ball.vel_x = hit_normal.x * current_magnitude;
                        //ball.vel_y = hit_normal.y * current_magnitude;

                        // Damage if breakable
                        if let Some(breakable) = breakables.get_mut(hit_ent) {
                            breakable.hp -= 1;
                            if breakable.hp <= 0 {
                                ents.delete(hit_ent)
                                    .expect("Failed to delete brick entity!");
                            }
                        }
                    }
                    // No collision
                    BallCollision::None => {}
                }
            }

            ball.vel_x = ball.vel_x.min(BALL_MAX_AXIS_VELOCITY).max(-BALL_MAX_AXIS_VELOCITY);
            ball.vel_y = ball.vel_y.min(BALL_MAX_AXIS_VELOCITY).max(-BALL_MAX_AXIS_VELOCITY);

            ball.last_pos = Point2f::new(transform.pos_x, transform.pos_y);

            if let Some(correction) = correction {
                transform.pos_x += correction.x;
                transform.pos_y += correction.y;
            }

            transform.pos_x += ball.vel_x;
            transform.pos_y += ball.vel_y;
        }
    }
}

pub fn check_ball_brick_collision<'a, I>(
    ent_id: u32,
    ball_center: Point2f,
    brick_bbs: I,
    paddles: &ReadStorage<'a, PlayerPaddleComponent>,
) -> BallCollision
where
    I: IntoIterator<Item = (&'a Entity, &'a BoundingBox)>,
{
    let mut collisions: Vec<(f32, BallCollision)> = Vec::new();
    for (box_ent, bb) in brick_bbs {
        if box_ent.id() == ent_id {
            continue;
        }

        let center_diff_x: f32 = ball_center.x - bb.center_x;
        let center_diff_y: f32 = ball_center.y - bb.center_y;

        let clamped_diff_x: f32 = center_diff_x.max(-bb.half_w).min(bb.half_w);
        let clamped_diff_y: f32 = center_diff_y.max(-bb.half_h).min(bb.half_h);

        let closest_x: f32 = bb.center_x + clamped_diff_x;
        let closest_y: f32 = bb.center_y + clamped_diff_y;

        let diff_x: f32 = ball_center.x - closest_x;
        let diff_y: f32 = ball_center.y - closest_y;

        let dist: f32 = (diff_x.powf(2.0) + diff_y.powf(2.0)).sqrt();
        if dist < BALL_BB_RADIUS {
            println!("HIT {}, {}", diff_x, diff_y);
            let dir = Vector2f::new(diff_x, diff_y).normalize();
            let collision_dir = AABBCollisionDirection::from_vector(dir);
            let is_paddle = paddles.get(*box_ent).is_some();

            if is_paddle {
                let x_dist_to_paddle_center = (ball_center.x - bb.center_x) / 2.0;
                let x_hit_percentage = x_dist_to_paddle_center / bb.half_w;

                // If we hit the paddle, just return that immediately. It takes priority over other collisions
                return BallCollision::Paddle { x_hit_percentage };
            } else {
                collisions.push((
                    dist,
                    BallCollision::Brick {
                        hit_normal: dir,
                        collision_dir,
                        hit_ent: *box_ent,
                    },
                ));
            }
        }
    }

    // Figure out which collision was closest and just return that
    collisions.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    if let Some((_, collision)) = collisions.first() {
        return *collision;
    }

    BallCollision::None
}

struct BoundingBoxSystem;

impl<'a> System<'a> for BoundingBoxSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, WorldBoundingBoxState>,
        ReadStorage<'a, TransformComponent>,
        WriteStorage<'a, BoundingBoxComponent>,
    );

    fn run(
        &mut self,
        (ents, mut world_bounding_boxes, transforms, mut bounding_boxes): Self::SystemData,
    ) {
        for (ent, transform, bounding_box) in (&ents, &transforms, &mut bounding_boxes).join() {
            let x = transform.pos_x - (transform.origin.x as f32 * transform.scale.x);
            let y = transform.pos_y - (transform.origin.y as f32 * transform.scale.y);
            let bb_x = bounding_box.x as f32 * transform.scale.x;
            let bb_y = bounding_box.y as f32 * transform.scale.y;
            let bb_w = bounding_box.w as f32 * transform.scale.x;
            let bb_h = bounding_box.h as f32 * transform.scale.y;

            let bb = BoundingBox {
                left_x: x + bb_x,
                right_x: x + bb_x + bb_w,
                top_y: y + bb_y,
                bottom_y: y + bb_y + bb_h,
                center_x: x + bb_x + (bb_w / 2.0),
                center_y: y + bb_y + (bb_h / 2.0),
                half_w: bb_w / 2.0,
                half_h: bb_h / 2.0,
            };

            bounding_box.bb = Some(bb.clone());
            world_bounding_boxes.boxes.insert(ent, bb);
        }

        // Clear any bounding boxes for deleted entities
        for (ent, _) in world_bounding_boxes.boxes.clone().iter() {
            if !ents.is_alive(*ent) {
                world_bounding_boxes.boxes.remove(ent);
            }
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
