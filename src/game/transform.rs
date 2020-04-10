use crate::game::{Point2f, Vector2f};
use specs::prelude::*;

// todo convert position to Vector2d

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
