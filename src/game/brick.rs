use crate::game::{
    audio::{self, AudioAssetDb, AudioAssetId},
    ball::BallComponent,
    level::LoadLevelEvent,
    physics::CollisionEvent,
    LevelState,
};
use shrev::EventChannel;
use specs::prelude::*;

pub const BRICK_DEFAULT_HP: i32 = 2;
pub const BRICK_SPRITE_WIDTH: u32 = 20;
pub const BRICK_SPRITE_HEIGHT: u32 = 13;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BrickType {
    Grey,
    Air,
    Green,
    Blue,
    Red,
    Purple,
}

pub struct BrickComponent {
    pub hp: i32,
    pub is_indestructible: bool,
}

impl BrickComponent {
    pub fn new(hp: i32) -> Self {
        BrickComponent {
            hp,
            is_indestructible: hp <= 0,
        }
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

    fn run(
        &mut self,
        (ents, audio_db, mut level, collision_events, mut bricks, balls): Self::SystemData,
    ) {
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
            if brick.is_indestructible {
                continue;
            }

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

        let mut is_level_complete = false;
        for (_, brick) in (&ents, &bricks).join() {
            if brick.is_indestructible {
                continue;
            }

            is_level_complete = true;
            break;
        }

        // If there are no more destructible bricks remaining, and no level load in progress, the level is complete
        if !is_level_complete && level.load_level_event.is_none() {
            let current_level = level.level;
            level.load_level_event = Some(LoadLevelEvent {
                level: current_level + 1,
            });
        }
    }
}
