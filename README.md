# Brickbreaker

## TODO
- [x] Move paddle with keyboard
    - A/D or Left/Right keys to move
- [x] Restrict paddle to screen
- [x] Ball entity that bounces around walls and on paddle
- [x] Brick entity and placement
- [x] Brick destruction
- [x] Ability for paddle to hold ball and shoot it with space (at start of game)
- [x] When ball falls behind paddle, restart game
- [x] Spawn ball on paddle when game starts
- [x] Implement support for sprite origins
    - Needs to affect rendering position and bounding box position
- [ ] Text Rendering for score
- [ ] Save a single high score
- [ ] Game Over screen with restart button
- [ ] Main menu
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