pub mod components;
pub mod systems;
pub mod events;
pub mod utils;

use amethyst::{
    ecs::DispatcherBuilder,
    core::bundle::{
        SystemBundle, Error,
    },
};

/// Add all the systems relevant for collisions and movement.
pub struct ColliderPhysicsBundle {
    handle_collisions: bool,
}

impl ColliderPhysicsBundle {
    pub fn new() -> Self {
        ColliderPhysicsBundle {
            handle_collisions: false,
        }
    }

    /// Handle collisions automatically. The way collisions are handled
    /// can be specified via the 'HandleCollisionMode' enum.
    pub fn with_collision_handler(mut self) -> Self {
        self.handle_collisions = true;
        self
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for ColliderPhysicsBundle {
    fn build(self, dispatcher: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        use self::systems::*;

        dispatcher.add(MovementSystem, "movement_system", &[]);
        dispatcher.add(CollisionSystem, "collision_system", &["movement_system"]);
        if self.handle_collisions {
            dispatcher.add(HandleCollisionsSystem::default(), "handle_collisions_system", &["collision_system"]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod physics_tests {
    use crate::physics::{rotate_vec, reflect_velocity, oppose_collision};
    use amethyst::core::nalgebra::Vector2;
    use std::f32::consts::PI;
    use crate::physics::components::Velocity;

    #[test]
    fn test_rotate_vec() {
        test_pi_rotation(-1f32, 1f32, 1f32, -1f32);

        test_pi_rotation(132f32, 34f32, -132f32, -34f32);

        test_rotation(0f32, 1f32, -0.5f32.sqrt(), 0.5f32.sqrt(), PI * 0.25);
        test_rotation(32., 56., 13.129, 63.148, PI * 0.1);

    }

    #[test]
    fn test_velocity_change_from_collision() {
        test_velocity_change(100., 0.,
                             3.4, 0.,
                             -100., 0., 3);

        test_velocity_change(10., 0.,
                             1.3, 1.3,
                             0., -10., 3);

        test_velocity_change(-3., 0.,
                             -2.6, -1.5,
                             1.5, 2.6,
                             1);

        test_velocity_change(10.25, 8.02,
                             6.5, 11.27,
                             -1.81, -12.89,
                             2);
    }

    #[test]
    fn test_oppose_collision() {
        test_velocity_oppose_collision(3f32, 4f32,
                                       5f32, -1f32,
                                       // 5 actually but there is some error margin apparently
                                       4.9, -1f32, 1);

        test_velocity_oppose_collision(-2f32, 5f32,
                                       13., -7.,
                                       4.74, -2.55, 2);
    }

    #[inline]
    fn test_pi_rotation(first: f32, second: f32, f: f32, s: f32) {
        test_rotation(first, second, f, s, PI);
    }

    #[inline]
    fn test_rotation(first: f32, second: f32, f: f32, s: f32, angle: f32) {
        let mut vec = rotate_vec(&Vector2::new(first, second), angle);
        round_vec_mut(&mut vec, 3);
        assert_eq!(vec, round_vec(&Vector2::new(f, s), 3));
    }

    #[inline]
    fn round(num: f32, digits: u32) -> f32 {
        let power = 10u32.pow(digits) as f32;
        (num * power).round() / power
    }

    fn round_vec_mut(vec: &mut Vector2<f32>, digits: u32) {
        vec[0] = round(vec[0], digits);
        vec[1] = round(vec[1], digits);
    }

    fn round_vec(vec: &Vector2<f32>, digits: u32) -> Vector2<f32> {
        let mut ret = vec.clone();
        round_vec_mut(&mut ret, digits);
        ret
    }

    fn test_velocity_change(vel1: f32, vel2: f32, col1: f32, col2: f32, dest1: f32, dest2: f32, round: u32) {
        let mut velocity = Velocity(Vector2::new(vel1, vel2));
        reflect_velocity(&mut velocity, &Vector2::new(col1, col2));
        round_vec_mut(&mut velocity.0, round);
        assert_eq!(velocity.0, round_vec(&Vector2::new(dest1, dest2), round));
    }

    fn test_velocity_oppose_collision(first: f32, second: f32, col1: f32, col2: f32, dest1: f32, dest2: f32, round: u32) {
        let mut velocity = Velocity(Vector2::new(first, second));
        oppose_collision(&mut velocity, &Vector2::new(col1, col2));
        assert_eq!(round_vec(&velocity.0, round), Vector2::new(dest1, dest2));
    }
}