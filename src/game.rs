use gfx::{color::*, input::*, renderer::*, sprite::*};
use nalgebra::Vector2;
use specs::prelude::*;
use std::collections::HashMap;

const DEFAULT_BALL_VELOCITY_X: f32 = 4.0;

pub struct GameState<'a, 'b> {
    pub world: World,
    pub tick_dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> GameState<'a, 'b> {
    pub fn new() -> GameState<'a, 'b> {
        let mut world = World::new();

        // Components
        world.register::<TransformComponent>();
        world.register::<VelocityComponent>();
        world.register::<SpriteComponent>();
        world.register::<PlayerPaddleComponent>();
        world.register::<BoundingBoxComponent>();

        // Resources
        world.insert(RenderCommander::new());
        world.insert(WorldBoundingBoxState::default());

        // Create paddle ent
        world
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
            .with(PlayerPaddleComponent {})
            .with(SpriteComponent {
                color: COLOR_WHITE,
                spritesheet_tex_id: 2,
                w: 128.0,
                h: 64.0,
                region: SpriteRegion {
                    x: 0,
                    y: 0,
                    w: 64,
                    h: 32,
                },
            })
            .build();

        // Create ball ent
        world
            .create_entity()
            .with(TransformComponent {
                pos_x: 64.0,
                pos_y: 200.0,
            })
            //.with(BoundingBoxComponent { x: 26, y: 26, w: 12, h: 12, bb: None })
            .with(VelocityComponent {
                vel_x: 0.5,
                vel_y: DEFAULT_BALL_VELOCITY_X,
            })
            .with(SpriteComponent {
                color: COLOR_WHITE,
                spritesheet_tex_id: 2,
                w: 64.0,
                h: 64.0,
                region: SpriteRegion {
                    x: 64,
                    y: 0,
                    w: 32,
                    h: 32,
                },
            })
            .build();

        // Create brick ents
        world
            .create_entity()
            .with(TransformComponent {
                pos_x: 64.0,
                pos_y: 100.0,
            })
            .with(BoundingBoxComponent {
                x: 0,
                y: 14,
                w: 64,
                h: 36,
                bb: None,
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
        world
            .create_entity()
            .with(TransformComponent {
                pos_x: 140.0,
                pos_y: 128.0,
            })
            .with(BoundingBoxComponent {
                x: 0,
                y: 14,
                w: 64,
                h: 36,
                bb: None,
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
        world
            .create_entity()
            .with(TransformComponent {
                pos_x: 200.0,
                pos_y: 60.0,
            })
            .with(BoundingBoxComponent {
                x: 0,
                y: 14,
                w: 64,
                h: 36,
                bb: None,
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

        let tick_dispatcher = DispatcherBuilder::new()
            .with(PlayerPaddleSystem, "player_paddle", &[])
            .with(BoundingBoxSystem, "bounding_box", &["player_paddle"])
            .with(BallPhysicsSystem, "ball_physics", &["bounding_box"])
            //.with(BreakableSystem, "breakable", &[])
            .with_thread_local(SpriteSystem {})
            .build();

        GameState {
            world,
            tick_dispatcher,
        }
    }
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
pub struct VelocityComponent {
    vel_x: f32,
    vel_y: f32,
}

impl Component for VelocityComponent {
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

pub struct PlayerPaddleComponent {}

impl Component for PlayerPaddleComponent {
    type Storage = VecStorage<Self>;
}

struct PlayerPaddleSystem;

impl<'a> System<'a> for PlayerPaddleSystem {
    type SystemData = (
        Read<'a, InputState>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, PlayerPaddleComponent>,
    );

    fn run(&mut self, (input, mut transforms, mut paddles): Self::SystemData) {
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
        }
    }
}

struct SpriteSystem;

impl<'a> System<'a> for SpriteSystem {
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
        WriteStorage<'a, VelocityComponent>,
        ReadStorage<'a, PlayerPaddleComponent>,
    );

    fn run(
        &mut self,
        (ents, world_bounding_boxes, mut transforms, mut velocities, paddles): Self::SystemData,
    ) {
        for (ent, transform, velocity) in (&ents, &mut transforms, &mut velocities).join() {
            // Check for wall collisions
            if transform.pos_x < 0.0 {
                transform.pos_x = 0.0;
                velocity.vel_x = -velocity.vel_x * 1.1;
            } else if transform.pos_x > (640.0 - 64.0) {
                transform.pos_x = 640.0 - 64.0;
                velocity.vel_x = -velocity.vel_x * 1.1;
            }

            // Check for ceiling collision
            if transform.pos_y < 0.0 {
                transform.pos_y = 0.0;
                velocity.vel_y = -velocity.vel_y * 1.1;
            } else if transform.pos_y > 480.0 {
                transform.pos_y = 480.0;
                velocity.vel_y = -velocity.vel_y * 1.1;
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
                if dist < 6.0 {
                    let offset_x = 6.0 - diff_x.abs();
                    let offset_y = 6.0 - diff_y.abs();

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
                        let temp_velocity = Vector2::<f32>::new(velocity.vel_x, velocity.vel_y);
                        velocity.vel_x = DEFAULT_BALL_VELOCITY_X * percentage * paddle_hit_force;
                        velocity.vel_y *= -1.0;

                        let normalized =
                            Vector2::<f32>::new(velocity.vel_x, velocity.vel_y).normalize();
                        let new_velocity = normalized * temp_velocity.magnitude();

                        velocity.vel_x = new_velocity.x;
                        velocity.vel_y = new_velocity.y;
                    } else {
                        // Ball hit a block or wall of some sort, just reflect velocity based on the collision direction
                        if (collision_dir == AABBCollisionDirection::Up)
                            || (collision_dir == AABBCollisionDirection::Down)
                        {
                            velocity.vel_y *= -1.0;

                            if collision_dir == AABBCollisionDirection::Up {
                                transform.pos_y -= offset_y;
                            } else if collision_dir == AABBCollisionDirection::Down {
                                transform.pos_y += offset_y;
                            }
                        } else if (collision_dir == AABBCollisionDirection::Left)
                            || (collision_dir == AABBCollisionDirection::Right)
                        {
                            velocity.vel_x *= -1.0;

                            if collision_dir == AABBCollisionDirection::Left {
                                transform.pos_x += offset_x;
                            } else if collision_dir == AABBCollisionDirection::Right {
                                transform.pos_x -= offset_x;
                            }
                        }
                    }
                }
            }

            // TODO Check for out of bounds (below paddle)

            velocity.vel_x = velocity.vel_x.min(8.0).max(-8.0);
            velocity.vel_y = velocity.vel_y.min(8.0).max(-8.0);

            transform.pos_x += velocity.vel_x;
            transform.pos_y += velocity.vel_y;
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
    }
}
