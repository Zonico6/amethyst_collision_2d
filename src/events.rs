use amethyst::{
    ecs::{
        Entity, Storage, storage::MaskedStorage,
    },
    core::{
        transform::Transform,
        nalgebra::Vector2,
    }
};

use crate::{
    components::{
        Collider2D,
    },
};

use std::ops::Deref;

/// Every Collision originates from an entity and has an associated path that goes from that
/// Entity in the direction of the collision. Namely, 'path' points to the center of the overlapping area.
#[derive(Debug)]
pub struct Collision {
    pub entity: Entity,
    pub path: Vector2<f32>,
}

impl Collision {
    pub fn new(entity: Entity, path: Vector2<f32>) -> Self {
        Self {
            entity,
            path,
        }
    }
}


/// On every registered collision, a CollisionEvent is sent to the corresponding EventChannel
#[derive(Debug)]
pub struct CollisionEvent {
    pub collisions: [Collision; 2],
}

impl CollisionEvent {
    pub fn new(first: Entity, second: Entity, collision_path_first: Vector2<f32>, collision_path_second: Vector2<f32>) -> Self {
        Self {
            collisions: [
                Collision::new(first, collision_path_first),
                Collision::new(second, collision_path_second),
            ]
        }
    }

    /// Generate a CollisionEvent from two entites and their Transforms.
    /// If there is no collision, None is returned.
    pub fn from_collision(first: Entity, second: Entity,
                      first_collider: &Collider2D, second_collider: &Collider2D,
                      first_transform: &Transform, second_transform: &Transform) -> Option<Self>
    {
        let first_scale = first_transform.scale();
        let first_collider = first_collider.scaled_by(first_scale[0], first_scale[1]);

        let second_scale = second_transform.scale();
        let second_collider = second_collider.scaled_by(second_scale[0], second_scale[1]);

        let translation = first_transform.translation();
        let pos = Vector2::new(translation[0], translation[1]);

        let other_translation = second_transform.translation();
        let other_pos = Vector2::new(other_translation[0], other_translation[1]);

        if let Some(coll_paths) =
        Collider2D::collision_paths(&first_collider, &pos,
                                    &second_collider, &other_pos) {
            Some(CollisionEvent::new(first, second, coll_paths.0, coll_paths.1))
        } else {
            None
        }
    }

    /// Generate a CollisionEvent from two entites. The relevant components are pulled from
    /// the Transform storage. If there is no collision, None is returned.
    pub fn from_collision_storage<C, T>(colliders: &Storage<'_, Collider2D, C>, transforms: &Storage<'_, Transform, T>,
                                    first: Entity, second: Entity) -> Option<Self>
        where
            C: Deref<Target = MaskedStorage<Collider2D>>,
            T: Deref<Target = MaskedStorage<Transform>>
    {
        if let (Some(first_coll), Some(second_coll),
            Some(first_trans), Some(second_trans)) = (
            colliders.get(first),
            colliders.get(second),
            transforms.get(first),
            transforms.get(second),
        ) {
            CollisionEvent::from_collision(first, second,
                                           first_coll, second_coll,
                                           first_trans, second_trans)
        } else {
            None
        }
    }
}
