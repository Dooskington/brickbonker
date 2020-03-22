use crate::{
    color::Color,
    mesh::Vertex,
    renderer::{GfxBuffer, GfxDescriptorSet, GfxMemory},
    texture::Texture,
};
use std::rc::Rc;

// The width and height, in pixels, of the font spritesheet
const SPRITESHEET_WIDTH: f32 = 256.0;
const SPRITESHEET_HEIGHT: f32 = 256.0;

// TODO
// sprite batch should have a spritesheet associated
// just create a spritebatch with a sprite sheet and take its data

pub struct SpriteBatch {
    sprites: Vec<SpriteRenderData>,
    is_dirty: bool,
    mesh: (Vec<Vertex>, Vec<u32>),
    texture: Rc<Texture>,
    descriptor_set: GfxDescriptorSet,

    // Buffers
    vertex_buffer: (GfxBuffer, GfxMemory),
    index_buffer: (GfxBuffer, GfxMemory),
    // TODO
    // Need to cleanup buffers!
}

impl SpriteBatch {
    pub fn new(
        texture: Rc<Texture>,
        descriptor_set: GfxDescriptorSet,
        vertex_buffer: (GfxBuffer, GfxMemory),
        index_buffer: (GfxBuffer, GfxMemory),
    ) -> SpriteBatch {
        SpriteBatch {
            sprites: Vec::new(),
            is_dirty: true,
            mesh: (vec![], vec![]),
            texture,
            descriptor_set,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn texture(&self) -> Rc<Texture> {
        self.texture.clone()
    }

    pub fn descriptor_set_ref(&self) -> &GfxDescriptorSet {
        &self.descriptor_set
    }

    pub fn vertex_buffer_ref(&self) -> &GfxBuffer {
        &self.vertex_buffer.0
    }

    pub fn vertex_buffer_mem_ref(&self) -> &GfxMemory {
        &self.vertex_buffer.1
    }

    pub fn index_buffer_ref(&self) -> &GfxBuffer {
        &self.index_buffer.0
    }

    pub fn index_buffer_mem_ref(&self) -> &GfxMemory {
        &self.index_buffer.1
    }

    pub fn construct_mesh(&mut self) {
        // No need to rebuild unless the screen has changed
        if !self.is_dirty {
            return;
        }

        // Clear the mesh so we can rebuild it
        self.mesh.0.clear();
        self.mesh.1.clear();
        self.is_dirty = false;

        // Construct the mesh from all sprites in the batch
        for spr_data in &self.sprites {
            let vertex_count: u32 = self.mesh.0.len() as u32;

            let color: [f32; 4] = spr_data.color.data();

            // TODO
            // need to pre compute these uvs

            let u: f32 = spr_data.region.x as f32 / SPRITESHEET_WIDTH;
            let v: f32 = spr_data.region.y as f32 / SPRITESHEET_HEIGHT;

            let width: f32 = spr_data.region.w as f32 / SPRITESHEET_WIDTH;
            let height: f32 = spr_data.region.h as f32 / SPRITESHEET_HEIGHT;

            let new_vertices: [Vertex; 4] = [
                // Top left
                Vertex {
                    position: [spr_data.x, spr_data.y, 0.0],
                    color,
                    uv: [u, v],
                },
                // Top right
                Vertex {
                    position: [spr_data.x + spr_data.w, spr_data.y, 0.0],
                    color,
                    uv: [u + width, v],
                },
                // Bottom right
                Vertex {
                    position: [spr_data.x + spr_data.w, spr_data.y + spr_data.h, 0.0],
                    color,
                    uv: [u + width, v + height],
                },
                // Bottom left
                Vertex {
                    position: [spr_data.x, spr_data.y + spr_data.h, 0.0],
                    color,
                    uv: [u, v + height],
                },
            ];

            let new_indices: [u32; 6] = [
                vertex_count,
                vertex_count + 1,
                vertex_count + 2,
                vertex_count + 2,
                vertex_count + 3,
                vertex_count,
            ];

            self.mesh.0.extend_from_slice(&new_vertices);
            self.mesh.1.extend_from_slice(&new_indices);
        }
    }

    pub fn mesh(&self) -> (&[Vertex], &[u32]) {
        (&self.mesh.0, &self.mesh.1)
    }

    pub fn add(&mut self, x: f32, y: f32, w: f32, h: f32, region: SpriteRegion, color: Color) {
        self.is_dirty = true;
        self.sprites.push(SpriteRenderData {
            x,
            y,
            w,
            h,
            color,
            region,
        });
    }

    pub fn clear(&mut self) {
        self.is_dirty = true;
        self.sprites = Vec::new();
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct SpriteRenderData {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: Color,
    pub region: SpriteRegion,
}

#[derive(Copy, Clone, PartialEq)]
pub struct SpriteRegion {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub struct SpriteSheet {
    w: u32,
    h: u32,
}

impl SpriteSheet {
    pub fn new(w: u32, h: u32) -> SpriteSheet {
        SpriteSheet { w, h }
    }
}
