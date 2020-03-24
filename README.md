# Brickbreaker

## TODO
- [x] Move paddle with keyboard
    - A/D or Left/Right keys to move
- [x] Restrict paddle to screen
- [x] Ball entity that bounces around walls and on paddle
- [x] Brick entity and placement
- [x] Brick destruction
- [x] Ability for paddle to hold ball and shoot it with space (at start of game)
- [ ] When ball falls behind paddle, restart game
- [x] Spawn ball on paddle when game starts
- [ ] Text Rendering for score
- [ ] Save a single high score
- [ ] Audio

TransformComponent
- Position

PlayerPaddleComponent
- Moves entity in response to player keyboard input
- Restricts paddle to screen

SpriteComponent
- Renders a sprite for an entity

BoundingBoxComponent

VelocityComponent
- Velocity

BreakableComponent