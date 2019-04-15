use amethyst::{
    ecs::{
        ReadStorage, WriteStorage, SystemData, Resources, shred::ResourceId, Entity
    },
    core::{
        nalgebra::Vector2,
    },
};

use crate::{
    components::{Velocity, HandleCollisionMode},
    events::Collision,
};

/// Bounces 'velocity' off of a surface, defined by it's'normal'.
/// Velocity is pointing towards the object beforehand and away from it afterwards.
pub fn reflect_mut(velocity: &mut Vector2<f32>, normal: &Vector2<f32>) {
    let dot = velocity.dot(&normal);
    let norms = velocity.norm() * normal.norm();
    let positive_angle = velocity[0] * normal[1] - velocity[1] * normal[0] >= 0f32;

    let mut angle = (dot / norms).acos();
    if !positive_angle {
        angle *= -1f32;
    }

    *velocity = -rotate_vec(velocity, angle * 2f32);
}

/// Changes the velocity vector to point away from the object.
/// The collision vector is the reversed surface-norm of the object; i.e. points towards the object.
/// The angle of the velocity vector after the calculation to the norm of the
pub fn reflect_velocity(velocity: &mut Velocity, collision: &Vector2<f32>) {
    reflect_mut(&mut velocity.0, &-collision);
}

fn rotate_vec(vec: &Vector2<f32>, angle: f32) -> Vector2<f32> {
    let sin = angle.sin();
    let cos = angle.cos();

    Vector2::new(
        vec[0] * cos - vec[1] * sin,
        vec[0] * sin + vec[1] * cos
    )
}

/// Velocity vector will pointing in the direction of the collision.
pub fn oppose_collision(velocity: &mut Velocity, other_collision: &Vector2<f32>) {
    velocity.0 = other_collision.normalize() * velocity.0.norm();
}

/// Modify components based on the collision mode and the collision paths.
pub fn handle_collision(collision: &Collision, other_collision: &Collision,
                        components: HandleCollisionComponents<'_>) {
    let velocity = components.velocity;

    match components.mode {
        HandleCollisionMode::Ignore => return,
        HandleCollisionMode::Reflect => reflect_velocity(velocity, &collision.path),
        HandleCollisionMode::Bounce(bounciness) => {
            reflect_velocity(velocity, &collision.path);
            velocity.0[0] = velocity.0[0] * bounciness;
            velocity.0[1] = velocity.0[1] * bounciness;
        }
        HandleCollisionMode::Oppose => {
            oppose_collision(velocity, &other_collision.path);
        }
    }
}

/// All relevant components for handling collisions.
///
/// Meant to be passed to the 'handle_collision' function.
pub struct HandleCollisionComponents<'a> {
    mode: &'a HandleCollisionMode,
    velocity: &'a mut Velocity,
}

type ModeStorage<'a> = ReadStorage<'a, HandleCollisionMode>;
type VelocityStorage<'a> = WriteStorage<'a, Velocity>;

/// All relevant component storages for handling collsions.
pub struct HandleCollisionStorages<'a> {
    modes: ModeStorage<'a>,
    velocities: VelocityStorage<'a>,
}

impl<'a> SystemData<'a> for HandleCollisionStorages<'a> {
    fn setup(res: &mut Resources) {
        <ModeStorage<'a> as SystemData>::setup(res);
        <VelocityStorage<'a> as SystemData>::setup(res);
    }

    fn fetch(res: &'a Resources) -> Self {
        let modes = <ModeStorage<'a> as SystemData<'a>>::fetch(res);
        let velocities = <VelocityStorage<'a> as SystemData<'a>>::fetch(res);

        HandleCollisionStorages {
            modes, velocities,
        }
    }

    fn reads() -> Vec<ResourceId> {
        let mut r = Vec::new();

        r.append(&mut <ModeStorage as SystemData>::reads());
        r.append(&mut <VelocityStorage as SystemData>::reads());

        r
    }

    fn writes() -> Vec<ResourceId> {
        let mut r = Vec::new();

        r.append(&mut <ModeStorage as SystemData>::writes());
        r.append(&mut <VelocityStorage as SystemData>::writes());

        r
    }
}

impl<'a> HandleCollisionStorages<'a> {
    pub fn get_components(&mut self, entity: Entity) -> Option<HandleCollisionComponents> {
        let (mode, velocity) = match (self.modes.get(entity), self.velocities.get_mut(entity)) {
            (Some(mode), Some(vel)) => (mode, vel),
            _ => return None
        };

        Some(HandleCollisionComponents {
            mode, velocity
        })
    }
}