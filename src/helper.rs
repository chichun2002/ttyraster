use crate::vectors::{Normal, Vec3};

pub struct Basis {
    pub right: Normal<Vec3>,
    pub up: Normal<Vec3>,
    pub forward: Normal<Vec3>,
}

pub fn lerp(a: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = (a - in_min) / (in_max - in_min);
    out_min * (1.0 - t) + out_max * t
}