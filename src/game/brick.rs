use specs::prelude::*;

pub const BRICK_DEFAULT_HP: i32 = 2;
pub const BRICK_SPRITE_WIDTH: u32 = 32;
pub const BRICK_SPRITE_HEIGHT: u32 = 16;

pub struct BreakableComponent {
    pub hp: i32,
}

impl Component for BreakableComponent {
    type Storage = VecStorage<Self>;
}
