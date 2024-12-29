use std::{i16, iter::Sum};

use cgmath::{BaseFloat, InnerSpace, Point3, Vector3};

#[derive(Clone)]
pub struct Plane<T> {
    pub pos: Vector3<T>,
    pub normal: Vector3<T>,
}

pub fn mean<T: BaseFloat + std::convert::From<i16>>(points: &Vec<Point3<T>>) -> Point3<T> {
    if points.len() > i16::MAX as usize {
        panic!("This method should not be used to calculate the mean for big vectors with a length of over 32767!");
    }
    let mut sum = Point3::<T> {
        x: T::zero(),
        y: T::zero(),
        z: T::zero(),
    };
    points.iter().for_each(|p| {
        sum.x += p.x;
        sum.y += p.y;
        sum.z += p.z;
    });
    let len = points.len() as i16;
    let len: T = len.into();
    sum /= len;
    sum
}

pub fn distance<T: BaseFloat + Sum>(point1: &Point3<T>, point2: &Point3<T>) -> T {
    T::sqrt(
        (0..3) // includes 0, excludes 3
            .map(|xyz| T::powi(point2[xyz] - point1[xyz], 2))
            .sum(),
    )
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
