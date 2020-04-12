use crate::game::{ball::BallComponent, transform::TransformComponent, Vector2d};
use gfx::input::{InputState, VirtualKeyCode};
use specs::prelude::*;

pub const PADDLE_HIT_BOX_WIDTH: f64 = 58.0;
pub const PADDLE_HIT_BOX_HEIGHT: f64 = 9.0;
pub const PADDLE_SPRITE_WIDTH: u32 = 64;
pub const PADDLE_SPRITE_HEIGHT: u32 = 32;
pub const PADDLE_SCALE_X: f32 = 1.0;
pub const PADDLE_SCALE_Y: f32 = 1.0;

pub struct PlayerPaddleComponent {
    pub held_ball_ent: Option<Entity>,
    pub held_ball_position: Vector2d,
    pub level_width: u32,
    movement_linear_velocity: Vector2d,
}

impl PlayerPaddleComponent {
    pub fn new(level_width: u32) -> Self {
        PlayerPaddleComponent {
            held_ball_ent: None,
            held_ball_position: Vector2d::zeros(),
            level_width,
            movement_linear_velocity: Vector2d::zeros(),
        }
    }
}

impl Component for PlayerPaddleComponent {
    type Storage = VecStorage<Self>;
}

pub struct PlayerPaddleSystem;

impl<'a> System<'a> for PlayerPaddleSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, InputState>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, PlayerPaddleComponent>,
        WriteStorage<'a, BallComponent>,
    );

    fn run(&mut self, (entities, input, mut transforms, mut paddles, mut balls): Self::SystemData) {
        for (transform, paddle) in (&mut transforms, &mut paddles).join() {
            let speed = 8.0;
            paddle.movement_linear_velocity = Vector2d::zeros();

            if input.is_key_held(VirtualKeyCode::A) || input.is_key_held(VirtualKeyCode::Left) {
                paddle.movement_linear_velocity.x -= speed;
            }

            if input.is_key_held(VirtualKeyCode::D) || input.is_key_held(VirtualKeyCode::Right) {
                paddle.movement_linear_velocity.x += speed;
            }

            transform.position += paddle.movement_linear_velocity;

            // Restrain paddle to the level
            let paddle_x_min = 4.0;
            let paddle_x_max = paddle.level_width as f64 - 4.0;
            let paddle_half_width = PADDLE_HIT_BOX_WIDTH / 2.0;
            if (transform.position.x - paddle_half_width) < paddle_x_min {
                transform.position.x = paddle_x_min + paddle_half_width;
            } else if (transform.position.x + paddle_half_width) > paddle_x_max {
                transform.position.x = paddle_x_max - paddle_half_width;
            }

            paddle.held_ball_position = transform.position
                + Vector2d::new(
                    0.0,
                    (-PADDLE_HIT_BOX_HEIGHT as f64 / 2.0)
                        - crate::game::ball::BALL_COLLIDER_RADIUS
                        - 2.0,
                );
        }

        // Handle paddles that are holding a ball
        for mut paddle in (&mut paddles).join() {
            if let Some(ball_ent) = paddle.held_ball_ent {
                if input.is_key_pressed(VirtualKeyCode::Space) {
                    paddle.held_ball_ent = None;

                    let ball = balls.get_mut(ball_ent).expect(
                        "Failed to set held_ball_ent position! Entity had no BallComponent!",
                    );

                    ball.holding_paddle_ent = None;
                    ball.velocity.linear = paddle.movement_linear_velocity * 0.5;
                    ball.velocity.linear.y = -crate::game::ball::BALL_DEFAULT_FORCE;
                }
            }
        }
    }
}
