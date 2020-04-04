use crate::game::*;
use std::collections::HashMap;
use specs::prelude::*;
use nalgebra::Vector2;
use nphysics2d::object::{Ground, DefaultBodySet, DefaultColliderSet, BodyStatus, RigidBodyDesc, DefaultColliderHandle, DefaultBodyHandle, BodyPartHandle, BodyHandle, ColliderHandle, ColliderDesc};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};
use nphysics2d::math::{Velocity};
use ncollide2d::shape::{Ball, Cuboid, ShapeHandle};

pub struct PhysicsState {
    mechanical_world: DefaultMechanicalWorld<f64>,
    geometrical_world: DefaultGeometricalWorld<f64>,
    bodies: DefaultBodySet<f64>,
    colliders: DefaultColliderSet<f64>,
    joint_constraints: DefaultJointConstraintSet<f64>,
    force_generators: DefaultForceGeneratorSet<f64>,
    ent_body_handles: HashMap<u32, DefaultBodyHandle>,
    ent_collider_handles: HashMap<u32, DefaultColliderHandle>,
}

impl PhysicsState {
    pub fn new() -> Self {
        let gravity = Vector2::new(0.0, -9.81);
        let mechanical_world = DefaultMechanicalWorld::new(gravity);
        let geometrical_world = DefaultGeometricalWorld::new();
        let bodies = DefaultBodySet::new();
        let colliders = DefaultColliderSet::new();
        let joint_constraints = DefaultJointConstraintSet::new();
        let force_generators = DefaultForceGeneratorSet::new();
        let body_handles = HashMap::new();
        let collider_handles = HashMap::new();

        PhysicsState {
            mechanical_world,
            geometrical_world,
            bodies,
            colliders,
            joint_constraints,
            force_generators,
            ent_body_handles: body_handles,
            ent_collider_handles: collider_handles,
        }
    }
}

#[derive(Debug)]
pub struct RigidbodyComponent {
    velocity: Velocity<f64>,
}

impl RigidbodyComponent {
    pub fn new() -> Self {
        RigidbodyComponent {
            velocity: Velocity::new(Vector2::zeros(), 0.0)
        }
    }
}

impl Component for RigidbodyComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(Debug)]
pub struct ColliderComponent {

}

impl Component for ColliderComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(Default)]
pub struct RigidbodySendPhysicsSystem {
    pub inserted_bodies: BitSet,
    pub modified_bodies: BitSet,
    pub removed_bodies: BitSet,
    pub modified_transforms: BitSet,
    pub transform_reader_id: Option<ReaderId<ComponentEvent>>,
    pub rigidbody_reader_id: Option<ReaderId<ComponentEvent>>,
}

impl<'a> System<'a> for RigidbodySendPhysicsSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, PhysicsState>,
        ReadStorage<'a, RigidbodyComponent>,
        ReadStorage<'a, TransformComponent>,
    );

    fn run(&mut self, (ents, mut physics, rigidbodies, transforms): Self::SystemData) {
        self.inserted_bodies.clear();
        self.modified_bodies.clear();
        self.removed_bodies.clear();
        self.modified_transforms.clear();

        // Process TransformComponent events into a bitset
        let transform_events = transforms.channel().read(self.transform_reader_id.as_mut().unwrap());
        for event in transform_events {
            match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    self.modified_transforms.add(*id);
                },
                _ => {}
            }
        }

        // Process RigidbodyComponent events into bitsets
        let rigidbody_events = rigidbodies.channel().read(self.rigidbody_reader_id.as_mut().unwrap());
        for event in rigidbody_events {
            match event {
                ComponentEvent::Inserted(id) => {
                    self.inserted_bodies.add(*id);
                },
                ComponentEvent::Modified(id) => {
                    self.modified_bodies.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    self.removed_bodies.add(*id);
                }
            }
        }

        // Handle inserted rigidbodies
        for (transform, rigidbody, ent_id) in (&transforms, &rigidbodies, &self.inserted_bodies).join() {
            if let Some(rb_handle) = physics.ent_body_handles.remove(&ent_id) {
                eprintln!("[RigidbodySendPhysicsSystem] Duplicate rigidbody found in physics world! Removing it. Entity Id = {}, Handle = {:?}", ent_id, rb_handle);
                physics.bodies.remove(rb_handle);
            }

            let ball = ShapeHandle::new(Ball::new(1.0));
            let rigid_body = RigidBodyDesc::new()
                .translation(Vector2::new(0.0, 0.0))
                .rotation(0.0)
                .gravity_enabled(false)
                .status(BodyStatus::Dynamic)
                .velocity(Velocity::linear(0.0, 5.0))
                .max_linear_velocity(10.0)
                .linear_motion_interpolation_enabled(true)
                .user_data(ent_id)
                .build();

            let rb_handle = physics.bodies.insert(rigid_body);
            physics.ent_body_handles.insert(ent_id, rb_handle);
            println!("[RigidbodySendPhysicsSystem] Inserted rigidbody. Entity Id = {}, Handle = {:?}", ent_id, rb_handle);
        }

        // Handle modified rigidbodies
        for (rigidbody, ent_id) in (&rigidbodies, &self.modified_bodies).join() {
            if let Some(rb_handle) = physics.ent_body_handles.get(&ent_id).cloned() {
                let rb = physics.bodies.rigid_body_mut(rb_handle).unwrap();
                rb.set_velocity(rigidbody.velocity);
                println!("[RigidbodySendPhysicsSystem] Modified rigidbody: {}", ent_id);
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to update rigidbody because it didn't exist! Entity Id = {}", ent_id);
            }
        }

        // Handle removed rigidbodies
        for ent_id in (&self.removed_bodies).join() {
            if let Some(rb_handle) = physics.ent_body_handles.remove(&ent_id) {
                physics.bodies.remove(rb_handle);
                println!("[RigidbodySendPhysicsSystem] Removed rigidbody. Entity Id = {}", ent_id);
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to remove rigidbody because it didn't exist! Entity Id = {}", ent_id);
            }

        }

        // Handle modified transforms
        for (transform, _, ent_id) in (&transforms, &rigidbodies, &self.modified_transforms).join() {
            if let Some(rb_handle) = physics.ent_body_handles.get(&ent_id).cloned() {
                let rb = physics.bodies.rigid_body_mut(rb_handle).unwrap();
                // TODO transform component should have it's own isometry already
                rb.set_position(nalgebra::Isometry2::translation(transform.pos_x as f64, transform.pos_y as f64));
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to update rigidbody because it didn't exist! Entity Id = {}", ent_id);
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.transform_reader_id = Some(
            WriteStorage::<TransformComponent>::fetch(&world).register_reader()
        );
        self.rigidbody_reader_id = Some(
            WriteStorage::<RigidbodyComponent>::fetch(&world).register_reader()
        );
    }
}

#[derive(Default)]
pub struct ColliderSendPhysicsSystem {
    pub inserted_colliders: BitSet,
    pub modified_colliders: BitSet,
    pub removed_colliders: BitSet,
    pub modified_transforms: BitSet,
    pub transform_reader_id: Option<ReaderId<ComponentEvent>>,
    pub collider_reader_id: Option<ReaderId<ComponentEvent>>,
}

impl<'a> System<'a> for ColliderSendPhysicsSystem {
    type SystemData = (
        ReadStorage<'a, ColliderComponent>,
    );

    fn run(&mut self, (colliders): Self::SystemData) {
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.transform_reader_id = Some(
            WriteStorage::<TransformComponent>::fetch(&world).register_reader()
        );
        self.collider_reader_id = Some(
            WriteStorage::<ColliderComponent>::fetch(&world).register_reader()
        );
    }
}

pub struct WorldStepPhysicsSystem;

impl<'a> System<'a> for WorldStepPhysicsSystem {
    type SystemData = (
        ReadStorage<'a, TransformComponent>,
    );

    fn run(&mut self, (transforms): Self::SystemData) {
    }
}

pub struct RigidbodyReceivePhysicsSystem;

impl<'a> System<'a> for RigidbodyReceivePhysicsSystem {
    type SystemData = (
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, RigidbodyComponent>,
    );

    fn run(&mut self, (mut transforms, mut rigidbodies): Self::SystemData) {
    }
}
