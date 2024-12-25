use std::fmt::{Display, Formatter};

use cgmath::num_traits::Num;

#[derive(Debug, Clone)]
pub struct Point3<T: Num + Display> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Num + Display + Clone> Point3<T> {
    pub fn set(&mut self, other: &Point3<T>) {
        self.x = other.x.clone();
        self.y = other.y.clone();
        self.z = other.z.clone();
    }
}

impl<T: Num + Clone + Display> From<[T; 3]> for Point3<T> {
    fn from(value: [T; 3]) -> Self {
        Self {
            x: value[0].clone(),
            y: value[1].clone(),
            z: value[2].clone(),
        }
    }
}

impl<T: Num + Clone + Display> Into<[T; 3]> for Point3<T> {
    fn into(self) -> [T; 3] {
        [self.x, self.y, self.z]
    }
}

impl<T: Num + Display> Display for Point3<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x:{}, y:{}, z:{})", self.x, self.y, self.z)
    }
}
