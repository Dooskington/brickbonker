use crate::game::{
    physics::{PhysicsState, RigidbodyComponent},
    transform::TransformComponent,
};
use gfx::{
    color::*,
    renderer::{Renderable, TextureId, Transparency},
    sprite::*,
    Point2f, Vector2f,
};
use specs::prelude::*;

#[derive(Default)]
pub struct RenderState {
    commands: Vec<gfx::renderer::RenderCommand>,
    bound_transparency: Transparency,
    bound_texture_id: TextureId,
    bound_layer: u8,
    bound_color: Color,
}

impl RenderState {
    pub fn new() -> Self {
        RenderState {
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
pub struct SpriteRenderSystem;

impl<'a> System<'a> for SpriteRenderSystem {
    type SystemData = (
        ReadExpect<'a, PhysicsState>,
        Write<'a, RenderState>,
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
