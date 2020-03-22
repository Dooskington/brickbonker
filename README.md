# Brickbreaker

## TODO
- [ ] Move paddle with keyboard
- [ ] Restrict paddle to screen
- [ ] Ball entity that bounces around walls and on paddle
- [ ] Brick entity
- [ ] Brick destruction
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