use crate::game::{audio::{self, AudioAssetId, AudioAssetDb}, ball::BallComponent, physics::CollisionEvent, LevelState};
use shrev::EventChannel;
use specs::prelude::*;

pub const BRICK_DEFAULT_HP: i32 = 2;
pub const BRICK_SPRITE_WIDTH: u32 = 32;
pub const BRICK_SPRITE_HEIGHT: u32 = 16;

pub struct BrickComponent {
    pub hp: i32,
}

impl BrickComponent {
    pub fn new(hp: i32) -> Self {
        BrickComponent { hp }
    }
}

impl Component for BrickComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct BrickSystem {
    collision_event_reader: Option<ReaderId<CollisionEvent>>,
}

impl<'a> System<'a> for BrickSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, AudioAssetDb>,
        Write<'a, LevelState>,
        Read<'a, EventChannel<CollisionEvent>>,
        WriteStorage<'a, BrickComponent>,
        ReadStorage<'a, BallComponent>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.collision_event_reader = Some(
            world
                .fetch_mut::<EventChannel<CollisionEvent>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (ents, audio_db, mut level, collision_events, mut bricks, balls): Self::SystemData) {
        let mut bricks_hit_this_tick: BitSet = BitSet::new();
        for event in collision_events.read(&mut self.collision_event_reader.as_mut().unwrap()) {
            // Get the entities involved in the event, ignoring it entirely if either of them are not an entity
            let (entity_a, entity_b) = {
                if event.entity_a.is_none() || event.entity_b.is_none() {
                    continue;
                }

                (event.entity_a.unwrap(), event.entity_b.unwrap())
            };

            // If the collision was between a brick and a ball entity, mark the brick as hit so we can damage it
            if bricks.get(entity_a).is_some() && balls.get(entity_b).is_some() {
                bricks_hit_this_tick.add(entity_a.id());
            }
        }

        for (ent, mut brick, _) in (&ents, &mut bricks, &bricks_hit_this_tick).join() {
            brick.hp -= 1;
            if brick.hp <= 0 {
                ents.delete(ent).unwrap();

                level.score += 100;

                // Pick and play one of the brick break audio clips
                let clip_id = {
                    use rand::Rng;
                    let roll: f32 = rand::thread_rng().gen();

                    if roll <= 0.5 {
                        AudioAssetId::SfxBrickBreak0
                    } else {
                        AudioAssetId::SfxBrickBreak1
                    }
                };

                audio::play(clip_id, &audio_db, false);
            }
        }
    }
}
