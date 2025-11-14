use crate::vectors::{Normalize, Normal, Vec3};
use std::ops::Deref;
#[derive(Debug, Copy, Clone)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl std::ops::Mul<Quaternion> for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
    }
}
impl Quaternion {
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Quaternion {
        Quaternion { w, x, y, z }
    }

    pub fn inverse(&self) -> Quaternion {
        let norm_squared = self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z;

        Quaternion {
            w: self.w / norm_squared,
            x: -self.x / norm_squared,
            y: -self.y / norm_squared,
            z: -self.z / norm_squared,
        }
    }

    pub fn conjugate(&self) -> Quaternion {
        Quaternion {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    pub fn transform(&self, v: Vec3) -> Vec3 {
        let p = Quaternion::new(0.0, v.x, v.y, v.z);
        let t = ((*self) * p) * self.inverse();
        Vec3 {
            x: t.x,
            y: t.y,
            z: t.z,
        }
    }
}

pub fn from_axis_angle(axis: Vec3, angle: f32) -> Quaternion {
    let axis = if axis.length() != 1.0 {
        axis.normalized()
    } else {
        axis
    };
    let half_angle = angle / 2.0;
    Quaternion {
        w: half_angle.cos(),
        x: half_angle.sin() * axis.x,
        y: half_angle.sin() * axis.y,
        z: half_angle.sin() * axis.z,
    }
}

pub fn rotation_between_vectors(from: &Normal<Vec3>, to: &Normal<Vec3>) -> Quaternion {
    let to = *to.deref();
    let from = *from.deref();

    let dot = from.dot(to);
    let axis = from * to;

    if dot > 0.9999 {
        return Quaternion::new(1.0, 0.0, 0.0, 0.0);
    } else if dot < -0.9999 {
        let mut fallback_axis = Vec3::new(1.0, 0.0, 0.0);
        if (from * fallback_axis).length_squared() < 0.0001 {
            fallback_axis = Vec3::new(0.0, 1.0, 0.0);
        }
        return from_axis_angle(from * fallback_axis, std::f32::consts::PI);
    }

    let angle = dot.acos();
    from_axis_angle(axis, angle)
}
