use crate::game::{Point2f, Vector2d, Vector2f};
use specs::prelude::*;

#[derive(Debug)]
pub struct TransformComponent {
    pub position: Vector2d,
    pub last_position: Vector2d,
    pub origin: Point2f,
    pub scale: Vector2f,
}

impl Component for TransformComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl TransformComponent {
    pub fn new(position: Vector2d, origin: Point2f, scale: Vector2f) -> Self {
        TransformComponent {
            position,
            last_position: position,
            origin,
            scale,
        }
    }
}

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            position: Vector2d::zeros(),
            last_position: Vector2d::zeros(),
            origin: Point2f::origin(),
            scale: Vector2f::new(1.0, 1.0),
        }
    }
}
