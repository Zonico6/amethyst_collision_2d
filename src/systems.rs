use amethyst::{
    ecs::{
        System, SystemData, Join,
        Read, Write, ReadStorage, WriteStorage, Entities,
        Entity, Resources,
    },
    core::{
        shrev::{
            EventChannel, ReaderId,
        },
        timing::Time,
        transform::Transform,
    },
};

use crate::{
    events::CollisionEvent,
    components::*,
    utils::{
        handle_collision, HandleCollisionStorages,
    },
};

use std::collections::HashSet;

/// Test for collisions and sent them to EventChannel<CollisionEvent>.s
pub struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, EventChannel<CollisionEvent>>,
        ReadStorage<'a, Collider2D>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, DeactivateCollider>,
        ReadStorage<'a, PassiveCollider>,
    );

    fn run(&mut self, (entities, mut channel, colliders, transforms, deactivations, passive): Self::SystemData) {
        let mut covered: HashSet<Entity> = HashSet::new();

        for (entity, collider, transform, _, _) in (&entities, &colliders, &transforms, !&deactivations, !&passive).join() {
            covered.insert(entity);

            for (other, other_collider, other_transform, _) in (&entities, &colliders, &transforms, !&deactivations).join() {
                if covered.contains(&other) {
                    continue
                }
                if let Some(event) = CollisionEvent::from_collision(entity, other,
                                                                    collider, other_collider,
                                                                    transform, other_transform) {
                    channel.single_write(event);
                }
            }
        }
    }
}

/// Based on the 'HandleCollisionMode' of an Entity. For example, if the collision mode is
/// 'Reflect', then the Entity performs an elastic collision. This can be turned off for an
/// Entity by either not registering a 'HandleCollisionMode' for that Entity or setting it to 'Ignore'.
#[derive(Default)]
pub struct HandleCollisionsSystem {
    reader: Option<ReaderId<CollisionEvent>>
}

impl<'a> System<'a> for HandleCollisionsSystem {
    type SystemData = (
        Read<'a, EventChannel<CollisionEvent>>,
        HandleCollisionStorages<'a>,
    );

    fn run(&mut self, (channel, mut handle): Self::SystemData) {
        for event in channel.read(self.reader.as_mut().unwrap()) {
            let collisions = (&event.collisions[0], &event.collisions[1]);

            if let Some(comps) = handle.get_components(collisions.0.entity) {
                handle_collision(collisions.0, collisions.1, comps);
            }

            if let Some(comps) = handle.get_components(collisions.1.entity) {
                handle_collision(collisions.1, collisions.0, comps);
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        let reader = res
            .fetch_mut::<EventChannel<CollisionEvent>>()
            .register_reader();

        self.reader = Some(reader);
    }
}

/// Update the entities positions based on their 'Velocity' component.
pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Velocity>,
        Read<'a, Time>,
    );

    fn run(&mut self, (mut transforms, velocities, time): Self::SystemData) {
        for (transform, velocity) in (&mut transforms, &velocities).join() {
            let delta = time.delta_seconds();

            let (velx, vely) = (velocity.0[0], velocity.0[1]);

            transform.translate_x(velx * delta);
            transform.translate_y(vely * delta);
        }
    }
}