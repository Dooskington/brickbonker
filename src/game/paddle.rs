use crate::game::{Vector2d, ball::BallComponent, transform::TransformComponent};
use gfx::input::{InputState, VirtualKeyCode};
use specs::prelude::*;

#[derive(Default)]
pub struct PlayerPaddleComponent {
    pub held_ball_ent: Option<Entity>,
    pub held_ball_pos_x: f32,
    pub held_ball_pos_y: f32,
}

impl Component for PlayerPaddleComponent {
    type Storage = VecStorage<Self>;
}

pub struct PlayerPaddleSystem;

impl<'a> System<'a> for PlayerPaddleSystem {
    type SystemData = (
        Read<'a, InputState>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, PlayerPaddleComponent>,
        WriteStorage<'a, BallComponent>,
    );

    fn run(&mut self, (input, mut transforms, mut paddles, mut balls): Self::SystemData) {
        for (transform, paddle) in (&mut transforms, &mut paddles).join() {
            let speed = 8.0;
            let mut movement_linear_velocity = Vector2d::zeros();

            if input.is_key_held(VirtualKeyCode::A) || input.is_key_held(VirtualKeyCode::Left) {
                movement_linear_velocity.x -= speed;
            }

            if input.is_key_held(VirtualKeyCode::D) || input.is_key_held(VirtualKeyCode::Right) {
                movement_linear_velocity.x += speed;
            }

            transform.position += movement_linear_velocity;

            /*
            paddle.held_ball_pos_x = transform.pos_x as f64;
            paddle.held_ball_pos_y =
                transform.pos_y - (PADDLE_BB_HEIGHT as f64 * PADDLE_SCALE_Y) - BALL_BB_RADIUS;
            */
        }

        // Handle paddles that are holding a ball
        /*
        for mut paddle in (&mut paddles).join() {
            if let Some(ball_ent) = paddle.held_ball_ent {
                let ball_transform = transforms.get_mut(ball_ent).expect(
                    "Failed to set held_ball_ent position! Entity had no TransformComponent!",
                );
                ball_transform.pos_x = paddle.held_ball_pos_x;
                ball_transform.pos_y = paddle.held_ball_pos_y;

                if input.is_key_pressed(VirtualKeyCode::Space) {
                    paddle.held_ball_ent = None;

                    let ball = balls.get_mut(ball_ent).expect(
                        "Failed to set held_ball_ent position! Entity had no BallComponent!",
                    );

                    ball.is_held = false;
                    ball.vel_x = paddle.vel_x * 0.5;
                    ball.vel_y = -DEFAULT_BALL_FORCE;
                }
            }
        }
        */
    }
}
