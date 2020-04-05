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
    pub lerp: f64,
    mechanical_world: DefaultMechanicalWorld<f64>,
    geometrical_world: DefaultGeometricalWorld<f64>,
    bodies: DefaultBodySet<f64>,
    colliders: DefaultColliderSet<f64>,
    joint_constraints: DefaultJointConstraintSet<f64>,
    force_generators: DefaultForceGeneratorSet<f64>,
    ent_body_handles: HashMap<u32, DefaultBodyHandle>,
    ent_collider_handles: HashMap<u32, DefaultColliderHandle>,
    ground_body_handle: DefaultBodyHandle,
}

impl PhysicsState {
    pub fn new() -> Self {
        let gravity = Vector2::new(0.0, -9.81);
        let mechanical_world = DefaultMechanicalWorld::new(gravity);
        let geometrical_world = DefaultGeometricalWorld::new();
        let mut bodies = DefaultBodySet::new();
        let colliders = DefaultColliderSet::new();
        let joint_constraints = DefaultJointConstraintSet::new();
        let force_generators = DefaultForceGeneratorSet::new();
        let body_handles = HashMap::new();
        let collider_handles = HashMap::new();
        let ground_body_handle = bodies.insert(Ground::new());

        PhysicsState {
            lerp: 0.0,
            mechanical_world,
            geometrical_world,
            bodies,
            colliders,
            joint_constraints,
            force_generators,
            ent_body_handles: body_handles,
            ent_collider_handles: collider_handles,
            ground_body_handle,
        }
    }

    pub fn step(&mut self) {
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators
        );
    }
}

#[derive(Debug)]
pub struct RigidbodyComponent {
    velocity: Velocity<f64>,
    handle: Option<DefaultBodyHandle>,
}

impl RigidbodyComponent {
    pub fn new(velocity: Vector2<f64>) -> Self {
        RigidbodyComponent {
            velocity: Velocity::new(velocity, 0.0),
            handle: None,
        }
    }
}

impl Component for RigidbodyComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(Debug)]
pub struct ColliderComponent {

}

impl ColliderComponent {
    pub fn new() -> Self {
        ColliderComponent {}
    }
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
        WriteExpect<'a, PhysicsState>,
        WriteStorage<'a, RigidbodyComponent>,
        ReadStorage<'a, TransformComponent>,
    );

    fn run(&mut self, (mut physics, mut rigidbodies, transforms): Self::SystemData) {
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
        for (transform, rigidbody, ent_id) in (&transforms, &mut rigidbodies, &self.inserted_bodies).join() {
            if let Some(rb_handle) = physics.ent_body_handles.remove(&ent_id) {
                eprintln!("[RigidbodySendPhysicsSystem] Duplicate rigidbody found in physics world! Removing it. Entity Id = {}, Handle = {:?}", ent_id, rb_handle);
                physics.bodies.remove(rb_handle);
            }

            let ball = ShapeHandle::new(Ball::new(1.0));
            let rigid_body = RigidBodyDesc::new()
                .translation(Vector2::new(transform.pos_x as f64 / 32.0, transform.pos_y as f64 / 32.0))
                .rotation(0.0)
                .gravity_enabled(false)
                .status(BodyStatus::Dynamic)
                .velocity(Velocity::linear(0.0, 0.0))
                .max_linear_velocity(50.0)
                .linear_motion_interpolation_enabled(true)
                .user_data(ent_id)
                .build();

            let rb_handle = physics.bodies.insert(rigid_body);
            rigidbody.handle = Some(rb_handle);
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
                //rb.set_position(nalgebra::Isometry2::translation(transform.pos_x as f64 / 32.0, transform.pos_y as f64 / 32.0));
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
        WriteExpect<'a, PhysicsState>,
        ReadStorage<'a, ColliderComponent>,
        ReadStorage<'a, TransformComponent>,
    );

    fn run(&mut self, (mut physics, colliders, transforms): Self::SystemData) {
        self.inserted_colliders.clear();
        self.modified_colliders.clear();
        self.removed_colliders.clear();
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

        // Process ColliderComponent events into bitsets
        let collider_events = colliders.channel().read(self.collider_reader_id.as_mut().unwrap());
        for event in collider_events {
            match event {
                ComponentEvent::Inserted(id) => {
                    self.inserted_colliders.add(*id);
                },
                ComponentEvent::Modified(id) => {
                    self.modified_colliders.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    self.removed_colliders.add(*id);
                }
            }
        }

        // Handle inserted colliders
        for (transform, collider, ent_id) in (&transforms, &colliders, &self.inserted_colliders).join() {
            if let Some(collider_handle) = physics.ent_collider_handles.remove(&ent_id) {
                eprintln!("[ColliderSendPhysicsSystem] Duplicate collider found in physics world! Removing it. Entity Id = {}, Handle = {:?}", ent_id, collider_handle);
                physics.colliders.remove(collider_handle);
            }

            // If this entity has a rigidbody, we need to attach the collider to it.
            // Otherwise we just attach it to the "ground".
            let (parent_body_handle, translation) = if let Some(rb_handle) = physics.ent_body_handles.get(&ent_id) {
                (rb_handle.clone(), Vector2::<f64>::zeros())
            } else {
                (physics.ground_body_handle.clone(), Vector2::<f64>::new(transform.pos_x as f64, transform.pos_y as f64))
            };

            let box_shape = ShapeHandle::new(Cuboid::new(Vector2::new(0.5, 0.5)));
            let box_collider = ColliderDesc::new(box_shape.clone())
                .density(0.0)
                .translation(translation)
                .set_ccd_enabled(true)
                .build(BodyPartHandle(parent_body_handle, 0));
            let collider_handle = physics.colliders.insert(box_collider);
            physics.ent_collider_handles.insert(ent_id, collider_handle);
            println!("[ColliderSendPhysicsSystem] Inserted collider. Entity Id = {}, Handle = {:?}", ent_id, collider_handle);
        }

        // Handle modified colliders
        for (rigidbody, ent_id) in (&colliders, &self.modified_colliders).join() {
            if let Some(collider_handle) = physics.ent_collider_handles.get(&ent_id).cloned() {
                println!("[ColliderSendPhysicsSystem] Modified collider: {}", ent_id);
            } else {
                eprintln!("[ColliderSendPhysicsSystem] Failed to update collider because it didn't exist! Entity Id = {}", ent_id);
            }
        }

        // Handle removed colliders
        for ent_id in (&self.removed_colliders).join() {
            if let Some(collider_handle) = physics.ent_collider_handles.remove(&ent_id) {
                physics.colliders.remove(collider_handle);
                println!("[ColliderSendPhysicsSystem] Removed collider. Entity Id = {}", ent_id);
            } else {
                eprintln!("[ColliderSendPhysicsSystem] Failed to remove collider because it didn't exist! Entity Id = {}", ent_id);
            }
        }

        // Handle modified transforms
        /*
        for (transform, _, ent_id) in (&transforms, &colliders, &self.modified_transforms).join() {
            if let Some(collider_handle) = physics.ent_body_handles.get(&ent_id).cloned() {
                let rb = physics.bodies.rigid_body_mut(rb_handle).unwrap();
                // TODO transform component should have it's own isometry already
                rb.set_position(nalgebra::Isometry2::translation(transform.pos_x as f64, transform.pos_y as f64));
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to update rigidbody because it didn't exist! Entity Id = {}", ent_id);
            }
        }
        */
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

#[derive(Default)]
pub struct WorldStepPhysicsSystem;

impl<'a> System<'a> for WorldStepPhysicsSystem {
    type SystemData = (
        WriteExpect<'a, PhysicsState>,
    );

    fn run(&mut self, physics: Self::SystemData) {
        let mut physics = physics.0;

        for (_, handle) in physics.ent_body_handles.iter() {
            if let Some(body) = physics.bodies.rigid_body(*handle) {
                let pos = body.position().translation.vector;
                //println!("physics step: before: {}, {}", pos.x, pos.y)
            }
        }

        physics.step();

        for (_, handle) in physics.ent_body_handles.iter() {
            if let Some(body) = physics.bodies.rigid_body(*handle) {
                let pos = body.position().translation.vector;
                //println!("physics step: after: {}, {}", pos.x, pos.y)
            }
        }

        for contact in physics.geometrical_world.contact_events() {
            println!("Got contact: {:?}", contact);
        }
    }
}

pub struct RigidbodyReceivePhysicsSystem;

impl<'a> System<'a> for RigidbodyReceivePhysicsSystem {
    type SystemData = (
        ReadExpect<'a, PhysicsState>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, RigidbodyComponent>,
    );

    fn run(&mut self, (physics, mut transforms, rigidbodies): Self::SystemData) {
        for (rigidbody, transform) in (&rigidbodies, &mut transforms).join() {
            if let Some(body) = physics.bodies.rigid_body(rigidbody.handle.unwrap()) {
                transform.last_pos_x = transform.pos_x;
                transform.last_pos_y = transform.pos_y;
                //println!("before: {}, {}", transform.last_pos_x, transform.last_pos_y);

                let pos = body.position().translation.vector;
                transform.pos_x = pos.x * 32.0;
                transform.pos_y = pos.y * 32.0;

                //println!("after: {}, {}", transform.pos_x, transform.pos_y);
            }
        }
    }
}
