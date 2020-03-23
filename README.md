# Brickbreaker

## TODO
- [x] Move paddle with keyboard
    - A/D or Left/Right keys to move
- [x] Restrict paddle to screen
- [x] Ball entity that bounces around walls and on paddle
- [x] Brick entity and placement
- [x] Brick destruction
- [ ] Ability for paddle to hold ball and shoot it with space (at start of game)
- [ ] When ball falls behind paddle, game over

TransformComponent
- Position
- Last Position

PlayerPaddleComponent
- Moves entity in response to player keyboard input
- Restricts paddle to screen

SpriteComponent
- Renders a sprite for an entity

ColliderComponent
- Bounding Box

VelocityComponent
- Velocity

BreakableComponent