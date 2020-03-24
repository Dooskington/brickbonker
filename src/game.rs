use gfx::{color::*, input::*, renderer::*, sprite::*};
use nalgebra::Vector2;
use specs::prelude::*;
use shrev::EventChannel;
use std::collections::HashMap;

const DEFAULT_BALL_FORCE: f32 = 5.0;
const DEFAULT_BRICK_HP: i32 = 3;
const PADDLE_WIDTH: f32 = 128.0;
const PADDLE_HEIGHT: f32 = 64.0;
const BALL_WIDTH: f32 = 64.0;
const BALL_HEIGHT: f32 = 64.0;
const BALL_BB_RADIUS: f32 = 7.0;

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

        // Resources
        world.insert(RenderCommander::new());
        world.insert(WorldBoundingBoxState::default());

        // Create paddle ent
        let paddle_ent = world
            .create_entity()
            .with(TransformComponent {
                pos_x: 64.0,
                pos_y: 425.0,
            })
            .with(BoundingBoxComponent {
                x: 16,
                y: 24,
                w: 96,
                h: 16,
                bb: None,
            })
            .with(PlayerPaddleComponent::default())
            .with(SpriteComponent {
                color: COLOR_WHITE,
                spritesheet_tex_id: 2,
                w: PADDLE_WIDTH,
                h: PADDLE_HEIGHT,
                region: SpriteRegion {
                    x: 0,
                    y: 0,
                    w: 64,
                    h: 32,
                },
            })
            .build();

        // Create brick ents
        for x in 0..9 {
            for y in 0..4 {
                world
                .create_entity()
                .with(TransformComponent {
                    pos_x: 32.0 + (x as f32 * 64.0),
                    pos_y: 32.0 + (y as f32 * 40.0),
                })
                .with(BoundingBoxComponent {
                    x: 0,
                    y: 14,
                    w: 64,
                    h: 36,
                    bb: None,
                })
                .with(BreakableComponent {
                    hp: DEFAULT_BRICK_HP
                })
                .with(SpriteComponent {
                    color: COLOR_WHITE,
                    spritesheet_tex_id: 2,
                    w: 64.0,
                    h: 64.0,
                    region: SpriteRegion {
                        x: 96,
                        y: 0,
                        w: 32,
                        h: 32,
                    },
                })
                .build();
            }
        }

        let mut tick_dispatcher = DispatcherBuilder::new()
            .with(BallPhysicsSystem, "ball_physics", &[])
            .with(PlayerPaddleSystem, "player_paddle", &[])
            .with(BoundingBoxSystem, "bounding_box", &["player_paddle", "ball_physics"])
            .with_thread_local(SpawnBallSystem::default())
            .with_thread_local(SpriteRenderSystem {})
            .build();

        tick_dispatcher.setup(&mut world);

        // Spawn the initial ball
        world.write_resource::<EventChannel<SpawnBallEvent>>().single_write(SpawnBallEvent {
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

#[derive(Debug, PartialEq, Eq)]
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
            let dot_product: f32 = vector.normalize().dot(&v);
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

    pub fn sprite(&mut self, x: f32, y: f32, w: f32, h: f32, region: SpriteRegion) {
        self.commands.push(gfx::renderer::RenderCommand {
            transparency: self.bound_transparency,
            shader_program_id: 1,
            tex_id: self.bound_texture_id,
            layer: self.bound_layer,
            data: Renderable::Sprite {
                x,
                y,
                w,
                h,
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
    pos_x: f32,
    pos_y: f32,
}

impl Component for TransformComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct BallComponent {
    vel_x: f32,
    vel_y: f32,
    is_held: bool,
}

impl Component for BallComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct BoundingBoxComponent {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
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
    pub w: f32,
    pub h: f32,
}

impl Component for SpriteComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct PlayerPaddleComponent {
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

            let speed = 8.0;
            let mut movement_x: f32 = 0.0;

            if is_moving_left {
                movement_x -= speed;
            }

            if is_moving_right {
                movement_x += speed;
            }

            transform.pos_x += movement_x;

            if transform.pos_x < 0.0 {
                transform.pos_x = 0.0;
            } else if transform.pos_x > (640.0 - 128.0) {
                transform.pos_x = 640.0 - 128.0;
            }

            paddle.held_ball_pos_x = transform.pos_x + (PADDLE_WIDTH / 2.0) - (BALL_WIDTH / 2.0);
            paddle.held_ball_pos_y = transform.pos_y - (BALL_HEIGHT / 2.0);
        }

        // Handle paddles that are holding a ball
        for mut paddle in (&mut paddles).join() {
            if let Some(ball_ent) = paddle.held_ball_ent {
                let ball_transform = transforms.get_mut(ball_ent).expect("Failed to set held_ball_ent position! Entity had no TransformComponent!");
                ball_transform.pos_x = paddle.held_ball_pos_x;
                ball_transform.pos_y = paddle.held_ball_pos_y;

                if input.is_key_pressed(VirtualKeyCode::Space) {
                    paddle.held_ball_ent = None;

                    let ball = balls.get_mut(ball_ent).expect("Failed to set held_ball_ent position! Entity had no BallComponent!");
                    ball.is_held = false;
                    ball.vel_x = 0.0;
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
                sprite.w,
                sprite.h,
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
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, BallComponent>,
        WriteStorage<'a, BreakableComponent>,
        ReadStorage<'a, PlayerPaddleComponent>,
    );

    fn run(
        &mut self,
        (ents, world_bounding_boxes, mut transforms, mut balls, mut breakables, paddles): Self::SystemData,
    ) {
        for (ent, transform, ball) in (&ents, &mut transforms, &mut balls).join() {
            if ball.is_held {
                continue;
            }

            // Check for wall collisions
            if transform.pos_x < 0.0 {
                transform.pos_x = 0.0;
                ball.vel_x = -ball.vel_x * 1.1;
            } else if transform.pos_x > (640.0 - 64.0) {
                transform.pos_x = 640.0 - 64.0;
                ball.vel_x = -ball.vel_x * 1.1;
            }

            // Check for ceiling collision
            if transform.pos_y < 0.0 {
                transform.pos_y = 0.0;
                ball.vel_y = -ball.vel_y * 1.1;
            } else if transform.pos_y > 480.0 {
                transform.pos_y = 480.0;
                ball.vel_y = -ball.vel_y * 1.1;
            }

            // Check for collisions with bounding boxes (including the paddle)
            let ball_center_x: f32 = transform.pos_x + 32.0;
            let ball_center_y: f32 = transform.pos_y + 32.0;

            for (box_ent, bb) in world_bounding_boxes.boxes.iter() {
                if box_ent.id() == ent.id() {
                    continue;
                }

                let center_diff_x: f32 = ball_center_x - bb.center_x;
                let center_diff_y: f32 = ball_center_y - bb.center_y;

                let clamped_diff_x: f32 = center_diff_x.max(-bb.half_w).min(bb.half_w);
                let clamped_diff_y: f32 = center_diff_y.max(-bb.half_h).min(bb.half_h);

                let closest_x: f32 = bb.center_x + clamped_diff_x;
                let closest_y: f32 = bb.center_y + clamped_diff_y;

                let diff_x: f32 = ball_center_x - closest_x;
                let diff_y: f32 = ball_center_y - closest_y;

                let dist: f32 = (diff_x.powi(2) + diff_y.powi(2)).sqrt();
                if dist < BALL_BB_RADIUS {
                    let offset_x = BALL_BB_RADIUS - diff_x.abs();
                    let offset_y = BALL_BB_RADIUS - diff_y.abs();

                    let is_paddle = paddles.get(*box_ent).is_some();
                    let collision_dir =
                        AABBCollisionDirection::from_vector(Vector2::<f32>::new(diff_x, diff_y));
                    if is_paddle
                        && ((collision_dir == AABBCollisionDirection::Up)
                            || (collision_dir == AABBCollisionDirection::Down))
                    {
                        // Ball hit the paddle, reflect y velocity and set x velocity based on hit point

                        let x_dist_to_paddle_center = (ball_center_x - bb.center_x) / 2.0;
                        let percentage = x_dist_to_paddle_center / bb.half_w;

                        let paddle_hit_force: f32 = 2.0;
                        let temp_velocity = Vector2::<f32>::new(ball.vel_x, ball.vel_y);
                        ball.vel_x = DEFAULT_BALL_FORCE * percentage * paddle_hit_force;
                        ball.vel_y *= -1.0;

                        let normalized =
                            Vector2::<f32>::new(ball.vel_x, ball.vel_y).normalize();
                        let new_velocity = normalized * temp_velocity.magnitude();

                        ball.vel_x = new_velocity.x;
                        ball.vel_y = new_velocity.y;
                    } else {
                        // Ball hit a block or wall of some sort, just reflect velocity based on the collision direction
                        if (collision_dir == AABBCollisionDirection::Up)
                            || (collision_dir == AABBCollisionDirection::Down)
                        {
                            ball.vel_y *= -1.0;

                            if collision_dir == AABBCollisionDirection::Up {
                                transform.pos_y -= offset_y;
                            } else if collision_dir == AABBCollisionDirection::Down {
                                transform.pos_y += offset_y;
                            }
                        } else if (collision_dir == AABBCollisionDirection::Left)
                            || (collision_dir == AABBCollisionDirection::Right)
                        {
                            ball.vel_x *= -1.0;

                            if collision_dir == AABBCollisionDirection::Left {
                                transform.pos_x -= offset_x;
                            } else if collision_dir == AABBCollisionDirection::Right {
                                transform.pos_x += offset_x;
                            }
                        }

                        // Damage if breakable
                        if let Some(breakable) = breakables.get_mut(*box_ent) {
                            breakable.hp -= 1;
                            if breakable.hp <= 0 {
                                ents.delete(*box_ent).expect("Failed to delete brick entity!");
                            }
                        }
                    }
                }
            }

            // TODO Check for out of bounds (below paddle)

            ball.vel_x = ball.vel_x.min(8.0).max(-8.0);
            ball.vel_y = ball.vel_y.min(8.0).max(-8.0);

            transform.pos_x += ball.vel_x;
            transform.pos_y += ball.vel_y;
        }
    }
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
            let bb = BoundingBox {
                left_x: transform.pos_x + bounding_box.x as f32,
                right_x: transform.pos_x + bounding_box.x as f32 + bounding_box.w as f32,
                top_y: transform.pos_y + bounding_box.y as f32,
                bottom_y: transform.pos_y + bounding_box.y as f32 + bounding_box.h as f32,
                center_x: transform.pos_x + bounding_box.x as f32 + (bounding_box.w as f32 / 2.0),
                center_y: transform.pos_y + bounding_box.y as f32 + (bounding_box.h as f32 / 2.0),
                half_w: bounding_box.w as f32 / 2.0,
                half_h: bounding_box.h as f32 / 2.0,
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
        self.spawn_ball_event_reader = Some(world.fetch_mut::<EventChannel<SpawnBallEvent>>().register_reader());
    }

    fn run(&mut self, (ents, lazy_updater, spawn_ball_events, mut paddles): Self::SystemData) {
        for event in spawn_ball_events.read(&mut self.spawn_ball_event_reader.as_mut().unwrap()) {
            let ent = ents.create();

            lazy_updater.insert(ent, TransformComponent {
                pos_x: event.pos_x,
                pos_y: event.pos_y,
            });

            lazy_updater.insert(ent, SpriteComponent {
                color: COLOR_WHITE,
                spritesheet_tex_id: 2,
                w: BALL_WIDTH,
                h: BALL_HEIGHT,
                region: SpriteRegion {
                    x: 64,
                    y: 0,
                    w: 32,
                    h: 32,
                },
            });

            lazy_updater.insert(ent, BallComponent {
                vel_x: event.vel_x,
                vel_y: event.vel_y,
                is_held: event.owning_paddle_ent.is_some(),
            });

            if let Some(paddle_ent) = event.owning_paddle_ent {
                let mut paddle = paddles.get_mut(paddle_ent).expect("Failed to spawn ball ent: owning_paddle_ent not found!");
                paddle.held_ball_ent = Some(ent);
            }

            println!("[EntitySpawnSystem] Spawned ball");
        }
    }
}
