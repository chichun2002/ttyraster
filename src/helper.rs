use crate::vectors::{Normal, Vec3};

pub struct Basis {
    pub right: Normal<Vec3>,
    pub up: Normal<Vec3>,
    pub forward: Normal<Vec3>,
}