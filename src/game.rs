use gfx::{sprite::*, color::*, renderer::*, input::*};
use specs::prelude::*;
use std::collections::HashMap;

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

        // Resources
        world.insert(RenderCommander::new());

        // Create paddle ent
        world.create_entity()
            .with(TransformComponent { pos_x: 64.0, pos_y: 425.0 })
            .with(SpriteComponent { color: COLOR_WHITE, spritesheet_tex_id: 2, w: 128.0, h: 64.0, region: SpriteRegion {
                x: 0,
                y: 0,
                w: 64,
                h: 32,
            }})
            .with(PlayerPaddleComponent { })
            .build();

        // Create ball ent
        world.create_entity()
            .with(TransformComponent { pos_x: 64.0, pos_y: 300.0})
            .with(SpriteComponent { color: COLOR_WHITE, spritesheet_tex_id: 2, w: 64.0, h: 64.0, region: SpriteRegion {
                x: 64,
                y: 0,
                w: 32,
                h: 32,
            }})
            .with(VelocityComponent { vel_x: -4.0, vel_y: -1.0 })
            .build();

        let tick_dispatcher = DispatcherBuilder::new()
            .with(PlayerPaddleSystem, "player_paddle", &[])
            .with(BallPhysicsSystem, "ball_physics", &[])
            .with_thread_local(SpriteSystem {})
            .build();

        GameState {
            world,
            tick_dispatcher,
        }
    }
}

pub struct PhysicsObjectState {
    pub bb_left_x: f32,
    pub bb_right_x: f32,
    pub bb_top_y: f32,
    pub bb_bottom_y: f32,
}

pub struct PhysicsState {
    pub objects: HashMap<u64, PhysicsObjectState>
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

pub struct PlayerPaddleComponent {

}

impl Component for PlayerPaddleComponent {
    type Storage = VecStorage<Self>;
}

struct PlayerPaddleSystem;

impl<'a> System<'a> for PlayerPaddleSystem {
    type SystemData = (Read<'a, InputState>, WriteStorage<'a, TransformComponent>, WriteStorage<'a, PlayerPaddleComponent>);

    fn run(&mut self, (input, mut transforms, mut paddles): Self::SystemData) {
        for (transform, paddle) in (&mut transforms, &mut paddles).join() {
            let is_moving_left = input.is_key_held(VirtualKeyCode::A) || input.is_key_held(VirtualKeyCode::Left);
            let is_moving_right = input.is_key_held(VirtualKeyCode::D) || input.is_key_held(VirtualKeyCode::Right);

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
    type SystemData = (Write<'a, RenderCommander>, ReadStorage<'a, TransformComponent>, ReadStorage<'a, SpriteComponent>);

    fn run(&mut self, (mut render, transforms, sprites): Self::SystemData) {
        for (transform, sprite) in (&transforms, &sprites).join() {
            render.bind_texture(sprite.spritesheet_tex_id);
            render.bind_color(sprite.color);
            render.sprite(transform.pos_x, transform.pos_y, sprite.w, sprite.h, sprite.region);
        }
    }
}

struct BallPhysicsSystem;

impl<'a> System<'a> for BallPhysicsSystem {
    type SystemData = (WriteStorage<'a, TransformComponent>, WriteStorage<'a, VelocityComponent>);

    fn run(&mut self, (mut transforms, mut velocities): Self::SystemData) {
        for (transform, velocity) in (&mut transforms, &mut velocities).join() {
            transform.pos_x += velocity.vel_x;
            transform.pos_y += velocity.vel_y;

            // Check for wall collisions
            if transform.pos_x < 0.0 {
                transform.pos_x = 0.0;
                velocity.vel_x = -velocity.vel_x * 1.1;
                //velocity.vel_y *= 1.025;
                velocity.vel_y *= 0.95;
            } else if transform.pos_x > (640.0 - 64.0) {
                transform.pos_x = 640.0 - 64.0;
                velocity.vel_x = -velocity.vel_x * 1.05;
                //velocity.vel_y *= 1.025;
                velocity.vel_y *= 0.95;
            }

            // Check for ceiling collision
            if transform.pos_y < 0.0 {
                transform.pos_y = 0.0;
                //velocity.vel_x *= 1.025;
                velocity.vel_x *= 0.95;
                velocity.vel_y = -velocity.vel_y * 1.1;
            } else if transform.pos_y > 480.0 {
                transform.pos_y = 480.0;
                //velocity.vel_x *= 1.025;
                velocity.vel_x *= 0.95;
                velocity.vel_y = -velocity.vel_y * 1.05;
            }

            // Check for paddle collision

            // Check for out of bounds (below paddle)

            // TODO
            // Check for brick collisions

            velocity.vel_x = velocity.vel_x.min(8.0);
            velocity.vel_y = velocity.vel_y.min(8.0);
        }
    }
}
