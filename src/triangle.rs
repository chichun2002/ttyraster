use crate::vectors::Vec3;

#[derive(Debug)]
pub struct Triangle {
    pub p1: Vec3,
    pub p2: Vec3,
    pub p3: Vec3,
}

impl Triangle {
    pub fn new(p1: Vec3, p2: Vec3, p3: Vec3) -> Triangle {
        Triangle { p1, p2, p3 }
    }
}