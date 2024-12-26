use cgmath::{BaseFloat, InnerSpace, Point3, Vector3};

pub struct Plane<T> {
    pub pos: Vector3<T>,
    pub normal: Vector3<T>,
}

pub fn signed_distance<T: BaseFloat>(point: &Point3<T>, plane: &Plane<T>) -> T {
    let dist = Vector3::<T> {
        x: point.x - plane.pos.x,
        y: point.y - plane.pos.y,
        z: point.z - plane.pos.z,
    };
    dist.dot(plane.normal)
}

pub fn point_vs_plane<T: BaseFloat>(point: &Point3<T>, plane: &Plane<T>) -> Classification {
    let dist = signed_distance(point, plane);
    if dist == T::zero() {
        return Classification::Intersects;
    } else if dist > T::zero() {
        return Classification::InFront;
    } else {
        return Classification::Behind;
    }
}

#[derive(PartialEq, Eq)]
pub enum Classification {
    /// behind the plane, opposite direction of the planes normal
    Behind,
    /// in front of the plane, in direction of the planes normal
    InFront,
    /// touches the plane
    Intersects,
}