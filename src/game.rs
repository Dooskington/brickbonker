use specs::prelude::*;

pub struct GameState {}

impl GameState {
    pub fn new() -> GameState {
        GameState {}
    }
}

#[derive(Debug)]
struct TransformComponent {
    pos_x: f32,
    pos_y: f32,
}

impl Component for TransformComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct VelocityComponent {
    vel_x: f32,
    vel_y: f32,
}

impl Component for VelocityComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct SpriteComponent {

}

impl Component for SpriteComponent {
    type Storage = VecStorage<Self>;
}

struct PlayerPaddleComponent {

}

impl Component for PlayerPaddleComponent {
    type Storage = VecStorage<Self>;
}