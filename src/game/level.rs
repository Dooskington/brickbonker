use specs::prelude::*;

pub const LEVEL_BRICKS_Y_OFFSET: f64 = 16.0;
pub const LEVEL_BRICKS_WIDTH: u32 = 10;
pub const LEVEL_BRICKS_HEIGHT: u32 = 5;

#[derive(Default)]
pub struct LevelState {
    pub score: u32,
    pub lives: u32,
    pub player_paddle_ent: Option<Entity>,
}

impl LevelState {
    pub fn new(player_paddle_ent: Option<Entity>) -> LevelState {
        LevelState {
            score: 0,
            lives: 3,
            player_paddle_ent,
        }
    }
}

// TODO LoadLevelEvent and handling for it

struct LoadLevelEvent;
