use amethyst::ecs::{
    Component, DenseVecStorage, NullStorage,
};
use amethyst::core::nalgebra::{
    Vector2,
};

#[derive(Clone)]
pub enum Shape {
    Rectangle {
        width: f32,
        height: f32,
    },
}

#[derive(Clone)]
pub struct Collider2D {
    pub offset: Vector2<f32>,
    pub shape: Shape,
}

impl Collider2D {
    pub fn rect(width: f32, height: f32, offset: Vector2<f32>) -> Self {
        Collider2D {
            offset,
            shape: Shape::Rectangle { width, height}
        }
    }

    pub fn rect_without_offset(width: f32, height: f32) -> Self {
        Collider2D::rect(width, height, Vector2::new(0., 0.))
    }


    pub fn scaled_by(&self, x: f32, y: f32) -> Collider2D {
        Collider2D::rect(self.width() * x, self.height() * y,
                         Vector2::new(self.offset.x * x, self.offset.y * y))
    }

    pub fn width(&self) -> f32 {
        match self.shape {
            Shape::Rectangle { width, .. } => width.clone()
        }
    }

    pub fn height(&self) -> f32 {
        match self.shape {
            Shape::Rectangle { height, .. } => height.clone()
        }
    }
}

/// Enables Collisions between this entity and any other entity
/// with a Collider2D component attached to it.
///
/// To avoid having to test collisions between every Collider2D,
/// we differentiate between active and passive collisions.
/// PassiveColliders don't test for collisions themselves, they are solely tested by others.
///
/// This way, we can avoid testing for collisions between two entities that
/// don't move and therefore can't ever collide.
///
/// Therefore, entities that are static within the scene should be marked with the
/// ['PassiveCollider'] component to improve performance.
///
/// You can also deactivate collisions for this collider by adding the
/// ['DeactivateCollider'] component to an entity.
impl Collider2D {
    pub fn collides_with(&self, self_pos: &Vector2<f32>, other: &Collider2D, other_pos: &Vector2<f32>) -> bool {
        self.collision(self_pos, other, other_pos).is_some()
    }

    pub fn collision(&self, self_pos: &Vector2<f32>, other: &Collider2D, other_pos: &Vector2<f32>) -> Option<Vector2<f32>> {
        let Shape::Rectangle { width, height } = self.shape;
        let Shape::Rectangle { width: other_width, height: other_height} = other.shape;

        let coll_center = self_pos + self.offset;
        let other_coll_center = other_pos + other.offset;

        if let (Some(coll_x), Some(coll_y)) = (
            overlap_center(coll_center[0], width, other_coll_center[0], other_width),
            overlap_center(coll_center[1], height, other_coll_center[1], other_height)
        ) {
            Some(Vector2::new(coll_x, coll_y))
        } else {
            None
        }
    }

    pub fn collision_paths(&self, self_pos: &Vector2<f32>, other: &Collider2D, other_pos: &Vector2<f32>)
        -> Option<(Vector2<f32>, Vector2<f32>)>
    {
        Collider2D::collision(self, self_pos, other, other_pos)
            .map(|collision|
                (collision - self_pos, collision - other_pos))
    }
}

#[derive(Clone)]
struct Overlap {
    pub start: f32,
    pub width: f32,
}
fn overlap(start: f32, width: f32, other_start: f32, other_width: f32) -> Option<Overlap> {
    // Widths need to be positive
    let start = start - width.min(0.);
    let other_start = other_start - width.min(0.);
    let width = width.abs();
    let other_width = other_width.abs();

    let end = start + width;
    let other_end = other_start + other_width;

    if end < other_start || start > other_end {
        return None
    }

    let ov_start;
    let ov_width;

    if start < other_start {
        ov_start = other_start;

        if end > other_end {
            ov_width = other_width
        } else {
            ov_width = end - other_start
        }
    } else {
        ov_start = start;

        if end < other_end {
            ov_width = width;
        } else {
            ov_width = other_end - start;
        }
    }

    return Some(Overlap {
        start: ov_start,
        width: ov_width,
    });
}

fn overlap_center(pos: f32, extent: f32, other_pos: f32, other_extent: f32) -> Option<f32> {
    let overlap = overlap(pos - extent * 0.5, extent,
                          other_pos - other_extent * 0.5, other_extent);

    overlap.map(|ov| (ov.start + ov.width * 0.5))
}

impl Component for Collider2D {
    type Storage = DenseVecStorage<Self>;
}

/// Disables collision testing for this entity entirely.
///
/// This component has no effect if the host-entity does not have a ['Collider2D'] component as well.
#[derive(Default)]
pub struct DeactivateCollider;
impl Component for DeactivateCollider {
    type Storage = NullStorage<Self>;
}

/// Restricts collision testing for an entity.
///
/// Entities with this component don't test for collisions themselves,
/// rather they get tested by other, active colliders.
///
/// That way, we can avoid testing for collisions between entities that can't ever collide anyway.
/// For example, because they are both static and don't move.
///
/// This component has no effect if the host-entity does not have a ['Collider2D'] component as well.
#[derive(Default)]
pub struct PassiveCollider;
impl Component for PassiveCollider {
    type Storage = NullStorage<Self>;
}

/// Makes an entity move.
pub struct Velocity(pub Vector2<f32>);
impl Component for Velocity {
    type Storage = DenseVecStorage<Self>;
}

/// Automatically handle collisions. The way it is handled is directed by the variant.
#[derive(Debug)]
pub enum HandleCollisionMode {
    /// Does not handle the collision. Should be used you need to handle it in a custom way.
    Ignore,
    /// Redirect the Velocity Vector with angle of entry
    /// to the bounce surface being equal to the angle of reflection
    Reflect,
    /// Reflects, but also shortens the velocity vector by a factor of the value
    Bounce(f32),
    /// Velocity points away from the collision partner
    Oppose,
}
impl Component for HandleCollisionMode {
    type Storage = DenseVecStorage<Self>;
}

impl Default for HandleCollisionMode {
    fn default() -> Self {
        HandleCollisionMode::Ignore
    }
}

#[cfg(test)]
mod test_collision {
    use crate::physics::components::overlap_center;

    #[test]
    fn test_overlap() {
        // No overlap -- First under second
        assert_eq!(overlap_center(20., 3., 30., 4.), None);
        // No overlap -- First on top of second
        assert_eq!(overlap_center(30., 11., 15., 6.), None);

        // First under second
        assert_eq!(overlap_center(90., 68., 130., 20.), Some(122.));
        // First on top of second
        assert_eq!(overlap_center(24.6, 3.4, 20., 6.6), Some(23.1));

        // First inside second
        assert_eq!(overlap_center(1.34, 2.2, -40.6543, 2000.5), Some(1.34));
        // Second inside first
        assert_eq!(overlap_center(-124.2345, 3456.32, -2.34, 45.2).map(|pos| (pos * 100.).round() / 100.), Some(-2.34));
    }
}