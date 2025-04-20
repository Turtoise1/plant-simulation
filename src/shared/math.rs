use std::{f64, i16, iter::Sum};

use bevy::math::Vec3;
use cgmath::{BaseFloat, InnerSpace, Point3, Vector3};

#[derive(Clone, Debug)]
pub struct Line<T> {
    pub pos: Vector3<T>,
    pub dir: Vector3<T>,
}

#[derive(Clone, Debug)]
pub struct Plane<T> {
    pub pos: Vector3<T>,
    pub normal: Vector3<T>,
}

/// Calculate the radius of a sphere given its volume
/// Formula: radius = (3 * volume / (4 * π))^(1/3)
pub fn radius_from_volume<T: BaseFloat>(volume: T) -> T {
    let four_thirds_pi = T::from(4.0 / 3.0).unwrap() * T::from(f64::consts::PI).unwrap();
    (volume / four_thirds_pi).powf(T::from(1.0 / 3.0).unwrap())
}

pub fn volume_from_radius<T: BaseFloat>(volume: T) -> T {
    let four_thirds_pi = T::from(4.0 / 3.0).unwrap() * T::from(f64::consts::PI).unwrap();
    four_thirds_pi * volume.powi(3)
}

pub fn mean<T: BaseFloat>(points: &Vec<Point3<T>>) -> Point3<T> {
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
    let len: T = T::from(points.len()).unwrap();
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

pub fn point_vs_plane<T: BaseFloat>(
    point: &Point3<T>,
    plane: &Plane<T>,
) -> Point2PlaneClassification {
    let dist = signed_distance(point, plane);
    if dist == T::zero() {
        return Point2PlaneClassification::Intersects;
    } else if dist > T::zero() {
        return Point2PlaneClassification::InFront;
    } else {
        return Point2PlaneClassification::Behind;
    }
}

#[derive(PartialEq, Eq)]
pub enum Point2PlaneClassification {
    /// behind the plane, opposite direction of the planes normal
    Behind,
    /// in front of the plane, in direction of the planes normal
    InFront,
    /// touches the plane
    Intersects,
}

pub fn line_plane_intersection<T: BaseFloat>(
    line: &Line<T>,
    plane: &Plane<T>,
) -> Line2PlaneClassification<T> {
    let line_2_plane = plane.pos - line.pos;
    let denominator = plane.normal.dot(line.dir);

    // If denominator is close to zero, the line is parallel to the plane
    if denominator.abs() < T::epsilon() {
        return Line2PlaneClassification::Parallel;
    }

    let t = plane.normal.dot(line_2_plane) / denominator;

    let intersection = line.pos + (line.dir * t);
    Line2PlaneClassification::Intersects(intersection)
}

#[derive(PartialEq, Eq)]
pub enum Line2PlaneClassification<T: BaseFloat> {
    /// line parallel to the plane
    Parallel,
    /// line intersects with the plane
    Intersects(Vector3<T>),
}

pub fn to_bevy_vec3(point: &Point3<f32>) -> Vec3 {
    Vec3::new(point.x, point.y, point.z)
}

pub fn to_point3(vec: &Vec3) -> Point3<f32> {
    Point3::<f32>::new(vec.x, vec.y, vec.z)
}
