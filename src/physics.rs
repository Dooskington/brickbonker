use crate::game::*;
use nalgebra::Vector2;
use ncollide2d::{
    pipeline::{CollisionGroups, ContactEvent},
    shape::{Ball, Cuboid, Shape, ShapeHandle},
};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::math::Velocity;
use nphysics2d::object::{
    BodyHandle, BodyPartHandle, BodyStatus, ColliderDesc, ColliderHandle, DefaultBodyHandle,
    DefaultBodySet, DefaultColliderHandle, DefaultColliderSet, Ground, RigidBodyDesc,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
use specs::prelude::*;
use shrev::EventChannel;
use std::collections::HashMap;

#[derive(Debug)]
pub enum CollisionType {
    Started,
    Stopped,
}

pub struct CollisionEvent {
    pub entity_a: Option<Entity>,
    pub collider_handle_a: DefaultColliderHandle,
    pub entity_b: Option<Entity>,
    pub collider_handle_b: DefaultColliderHandle,
    pub normal: Vector2<f64>,
    pub depth: f64,
    pub ty: CollisionType,
}

pub struct PhysicsState {
    pub lerp: f64,
    pub bodies: DefaultBodySet<f64>,
    pub colliders: DefaultColliderSet<f64>,
    mechanical_world: DefaultMechanicalWorld<f64>,
    geometrical_world: DefaultGeometricalWorld<f64>,
    joint_constraints: DefaultJointConstraintSet<f64>,
    force_generators: DefaultForceGeneratorSet<f64>,
    ent_body_handles: HashMap<Entity, DefaultBodyHandle>,
    ent_collider_handles: HashMap<Entity, DefaultColliderHandle>,
    ground_body_handle: DefaultBodyHandle,
}

impl PhysicsState {
    pub fn new() -> Self {
        let mut bodies = DefaultBodySet::new();
        let colliders = DefaultColliderSet::new();

        let gravity = Vector2::new(0.0, -9.81);
        let mut mechanical_world = DefaultMechanicalWorld::new(gravity);
        mechanical_world
            .integration_parameters
            .max_ccd_position_iterations = 10;

        mechanical_world
            .integration_parameters
            .max_ccd_substeps = 2;

        let geometrical_world = DefaultGeometricalWorld::new();
        let joint_constraints = DefaultJointConstraintSet::new();
        let force_generators = DefaultForceGeneratorSet::new();
        let body_handles = HashMap::new();
        let collider_handles = HashMap::new();
        let ground_body_handle = bodies.insert(Ground::new());

        PhysicsState {
            lerp: 0.0,
            bodies,
            colliders,
            mechanical_world,
            geometrical_world,
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
            &mut self.force_generators,
        );
    }
}

#[derive(Debug)]
pub struct RigidbodyComponent {
    pub velocity: Velocity<f64>,
    pub last_velocity: Velocity<f64>,
    pub handle: Option<DefaultBodyHandle>,
    pub max_linear_velocity: f64,
}

impl RigidbodyComponent {
    pub fn new(velocity: Vector2<f64>) -> Self {
        let velocity = Velocity::new(velocity, 0.0);
        RigidbodyComponent {
            velocity,
            last_velocity: velocity,
            handle: None,
            max_linear_velocity: 100.0,
        }
    }
}

impl Component for RigidbodyComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

pub struct ColliderComponent {
    pub shape: ShapeHandle<f64>,
    pub offset: Vector2<f64>,
    pub collision_groups: CollisionGroups,
}

impl ColliderComponent {
    pub fn new<S: Shape<f64>>(shape: S, offset: Vector2<f64>, collision_groups: CollisionGroups) -> Self {
        ColliderComponent {
            shape: ShapeHandle::new(shape),
            offset,
            collision_groups,
        }
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
        Entities<'a>,
        WriteExpect<'a, PhysicsState>,
        WriteStorage<'a, RigidbodyComponent>,
        ReadStorage<'a, TransformComponent>,
    );

    fn run(&mut self, (entities, mut physics, mut rigidbodies, transforms): Self::SystemData) {
        self.inserted_bodies.clear();
        self.modified_bodies.clear();
        self.removed_bodies.clear();
        self.modified_transforms.clear();

        // Process TransformComponent events into a bitset
        let transform_events = transforms
            .channel()
            .read(self.transform_reader_id.as_mut().unwrap());
        for event in transform_events {
            match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    self.modified_transforms.add(*id);
                }
                _ => {}
            }
        }

        // Process RigidbodyComponent events into bitsets
        let rigidbody_events = rigidbodies
            .channel()
            .read(self.rigidbody_reader_id.as_mut().unwrap());
        for event in rigidbody_events {
            match event {
                ComponentEvent::Inserted(id) => {
                    self.inserted_bodies.add(*id);
                }
                ComponentEvent::Modified(id) => {
                    self.modified_bodies.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    self.removed_bodies.add(*id);
                }
            }
        }

        // Handle inserted rigidbodies
        for (ent, transform, rigidbody, ent_id) in
            (&entities, &transforms, &mut rigidbodies, &self.inserted_bodies).join()
        {
            if let Some(rb_handle) = physics.ent_body_handles.remove(&ent) {
                eprintln!("[RigidbodySendPhysicsSystem] Duplicate rigidbody found in physics world! Removing it. Entity Id = {}, Handle = {:?}", ent_id, rb_handle);
                physics.bodies.remove(rb_handle);
            }

            let rigid_body = RigidBodyDesc::new()
                .translation(Vector2::new(transform.pos_x / 32.0, transform.pos_y / 32.0))
                .rotation(0.0)
                .gravity_enabled(false)
                .status(BodyStatus::Dynamic)
                .velocity(rigidbody.velocity)
                .max_linear_velocity(rigidbody.max_linear_velocity)
                .mass(1.0)
                //.kinematic_translations(Vector2::new(true, true))
                .user_data(ent)
                .build();

            let rb_handle = physics.bodies.insert(rigid_body);
            rigidbody.handle = Some(rb_handle);
            physics.ent_body_handles.insert(ent, rb_handle);
            println!(
                "[RigidbodySendPhysicsSystem] Inserted rigidbody. Entity Id = {}, Handle = {:?}",
                ent_id, rb_handle
            );
        }

        // Handle modified rigidbodies
        for (ent, rigidbody, ent_id) in (&entities, &rigidbodies, &self.modified_bodies).join() {
            if let Some(rb_handle) = physics.ent_body_handles.get(&ent).cloned() {
                let rb = physics.bodies.rigid_body_mut(rb_handle).unwrap();
                let rb_velocity = rb.velocity();
                if (rb_velocity.linear != rigidbody.velocity.linear) || (rb_velocity.angular != rigidbody.velocity.angular) {
                    rb.set_velocity(rigidbody.velocity);
                    println!(
                        "[RigidbodySendPhysicsSystem] Modified rigidbody: {}, new vel: {:?}",
                        ent_id, rigidbody.velocity,
                    );
                }
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to update rigidbody because it didn't exist! Entity Id = {}", ent_id);
            }
        }

        // Handle removed rigidbodies
        for (ent, _) in (&entities, &self.removed_bodies).join() {
            if let Some(rb_handle) = physics.ent_body_handles.remove(&ent) {
                physics.bodies.remove(rb_handle);
                println!(
                    "[RigidbodySendPhysicsSystem] Removed rigidbody. Entity Id = {}",
                    ent.id()
                );
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to remove rigidbody because it didn't exist! Entity Id = {}", ent.id());
            }
        }

        // Handle modified transforms
        for (ent, transform, _, _) in (&entities, &transforms, &rigidbodies, &self.modified_transforms).join()
        {
            if let Some(rb_handle) = physics.ent_body_handles.get(&ent).cloned() {
            let rb = physics.bodies.rigid_body_mut(rb_handle).unwrap();
            // TODO transform component should have it's own isometry already
            rb.set_position(nalgebra::Isometry2::translation(transform.pos_x as f64 / 32.0, transform.pos_y as f64 / 32.0));
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to update rigidbody because it didn't exist! Entity Id = {}", ent.id());
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.transform_reader_id =
            Some(WriteStorage::<TransformComponent>::fetch(&world).register_reader());
        self.rigidbody_reader_id =
            Some(WriteStorage::<RigidbodyComponent>::fetch(&world).register_reader());
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
        Entities<'a>,
        WriteExpect<'a, PhysicsState>,
        ReadStorage<'a, ColliderComponent>,
        ReadStorage<'a, TransformComponent>,
    );

    fn run(&mut self, (entities, mut physics, colliders, transforms): Self::SystemData) {
        self.inserted_colliders.clear();
        self.modified_colliders.clear();
        self.removed_colliders.clear();
        self.modified_transforms.clear();

        // Process TransformComponent events into a bitset
        let transform_events = transforms
            .channel()
            .read(self.transform_reader_id.as_mut().unwrap());
        for event in transform_events {
            match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    self.modified_transforms.add(*id);
                }
                _ => {}
            }
        }

        // Process ColliderComponent events into bitsets
        let collider_events = colliders
            .channel()
            .read(self.collider_reader_id.as_mut().unwrap());
        for event in collider_events {
            match event {
                ComponentEvent::Inserted(id) => {
                    self.inserted_colliders.add(*id);
                }
                ComponentEvent::Modified(id) => {
                    self.modified_colliders.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    self.removed_colliders.add(*id);
                }
            }
        }

        // Handle inserted colliders
        for (ent, transform, collider, _) in
            (&entities, &transforms, &colliders, &self.inserted_colliders).join()
        {
            if let Some(collider_handle) = physics.ent_collider_handles.remove(&ent) {
                eprintln!("[ColliderSendPhysicsSystem] Duplicate collider found in physics world! Removing it. Entity Id = {}, Handle = {:?}", ent.id(), collider_handle);
                physics.colliders.remove(collider_handle);
            }

            // If this entity has a rigidbody, we need to attach the collider to it.
            // Otherwise we just attach it to the "ground".
            let (parent_body_handle, translation) =
                if let Some(rb_handle) = physics.ent_body_handles.get(&ent) {
                    (rb_handle.clone(), Vector2::<f64>::zeros())
                } else {
                    let ratio = 1.0 / 32.0;
                    (
                        physics.ground_body_handle.clone(),
                        Vector2::new(transform.pos_x, transform.pos_y) * ratio,
                    )
                };

            let collider = ColliderDesc::new(collider.shape.clone())
                .density(0.0)
                .translation(translation)
                .margin(0.02)
                .ccd_enabled(true)
                .collision_groups(collider.collision_groups.clone())
                .user_data(ent)
                .build(BodyPartHandle(parent_body_handle, 0));
            let collider_handle = physics.colliders.insert(collider);
            physics.ent_collider_handles.insert(ent, collider_handle);
            println!(
                "[ColliderSendPhysicsSystem] Inserted collider. Entity Id = {}, Handle = {:?}",
                ent.id(), collider_handle
            );
        }

        // Handle modified colliders
        for (ent, _, _) in (&entities, &colliders, &self.modified_colliders).join() {
            if let Some(_) = physics.ent_collider_handles.get(&ent).cloned() {
                // TODO
                println!("[ColliderSendPhysicsSystem] Modified collider: {}", ent.id());
            } else {
                eprintln!("[ColliderSendPhysicsSystem] Failed to update collider because it didn't exist! Entity Id = {}", ent.id());
            }
        }

        // Handle removed colliders
        for (ent, _) in (&entities, &self.removed_colliders).join() {
            if let Some(collider_handle) = physics.ent_collider_handles.remove(&ent) {
                physics.colliders.remove(collider_handle);
                println!(
                    "[ColliderSendPhysicsSystem] Removed collider. Entity Id = {}",
                    ent.id()
                );
            } else {
                eprintln!("[ColliderSendPhysicsSystem] Failed to remove collider because it didn't exist! Entity Id = {}", ent.id());
            }
        }

        // Handle modified transforms
        /*
        for (ent, transform, _, _) in (&entities, &transforms, &colliders, &self.modified_transforms).join() {
            if let Some(body_handle) = physics.ent_body_handles.get(&ent).cloned() {
                let rb = physics.bodies.rigid_body_mut(body_handle).unwrap();
                // TODO transform component should have it's own isometry already
                //rb.set_position(nalgebra::Isometry2::translation(transform.pos_x as f64, transform.pos_y as f64));
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to update rigidbody because it didn't exist! Entity Id = {}", ent.id());
            }
        }
        */
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.transform_reader_id =
            Some(WriteStorage::<TransformComponent>::fetch(&world).register_reader());
        self.collider_reader_id =
            Some(WriteStorage::<ColliderComponent>::fetch(&world).register_reader());
    }
}

#[derive(Default)]
pub struct WorldStepPhysicsSystem;

impl<'a> System<'a> for WorldStepPhysicsSystem {
    type SystemData = (WriteExpect<'a, PhysicsState>, WriteExpect<'a, EventChannel<CollisionEvent>>);

    fn run(&mut self, (mut physics, mut collision_events): Self::SystemData) {
        physics.step();

        for event in physics.geometrical_world.contact_events() {
            let collision_event = match event {
                ContactEvent::Started(handle1, handle2) => {
                    println!("ContactEvent::Started {:?}, {:?}", handle1, handle2);
                    if let Some(rb) = physics.bodies.rigid_body(*handle1) {
                        //println!("1: {:?}", rb.velocity());
                    }

                    if let Some(rb) = physics.bodies.rigid_body(*handle2) {
                        //println!("2: {:?}", rb.velocity());
                    }

                    for (h1, _, h2, _, _, _) in physics.geometrical_world.contact_pairs(&physics.colliders, false) {
                        //println!("contact pair: {:?} and {:?}", h1, h2);
                    }

                    if let Some((handle_a, collider_a, handle_b, collider_b, _, manifold)) =
                        physics.geometrical_world.contact_pair(
                            &physics.colliders,
                            *handle1,
                            *handle2,
                            false,
                        )
                    {
                        if manifold.len() == 0 {
                            println!("No contacts in manifold!");
                            //continue;
                        }

                        let entity_a = collider_a.user_data().unwrap().downcast_ref::<Entity>().cloned();
                        let entity_b = collider_b.user_data().unwrap().downcast_ref::<Entity>().cloned();
                        let depth = manifold.deepest_contact().unwrap().contact.depth;

                        println!("depth: {}", depth);
                        let normal = manifold.deepest_contact().unwrap().contact.normal.into_inner();

                        Some(CollisionEvent {
                            entity_a,
                            collider_handle_a: handle_a,
                            entity_b,
                            collider_handle_b: handle_b,
                            normal,
                            depth,
                            ty: CollisionType::Started,
                        })
                    } else {
                        panic!("No contact pair found for collision!");

                        None
                    }
                }
                ContactEvent::Stopped(_handle1, _handle2) => {
                    // TODO
                    None
                }
            };

            if let Some(ev) = collision_event {
                collision_events.single_write(ev);
            }
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

    fn run(&mut self, (physics, mut transforms, mut rigidbodies): Self::SystemData) {
        for (mut rigidbody, transform) in (&mut rigidbodies, &mut transforms).join() {
            if let Some(body) = physics.bodies.rigid_body(rigidbody.handle.unwrap()) {
                transform.last_pos_x = transform.pos_x;
                transform.last_pos_y = transform.pos_y;
                rigidbody.last_velocity = rigidbody.velocity.clone();

                let pos = body.position().translation.vector;
                transform.pos_x = pos.x * 32.0;
                transform.pos_y = pos.y * 32.0;

                rigidbody.velocity = body.velocity().clone();
                if rigidbody.last_velocity.linear != rigidbody.velocity.linear {
                    //println!("velocity change! {:?} to {:?}", rigidbody.last_velocity.linear, rigidbody.velocity.linear);
                }

                //println!("velocity: {:?}", rigidbody.velocity.linear);
            }
        }
    }
}