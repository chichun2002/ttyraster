#![allow(dead_code)]

use crate::helper::Basis;
use crate::quaternion::{rotation_between_vectors, Quaternion};
use crate::screen::Screen;
use crate::vectors::{Normal, Vec2, Vec3};

pub struct Projection {
    scale: f32,
    cx: f32,
    cy: f32,
}

impl Projection {
    #[inline]
    pub fn project(&self, v: Vec3) -> (Vec2, f32) {
        let s = self.scale / v.z;
        (Vec2::new(v.x * s + self.cx, self.cy - v.y * s), v.z)
    }
}
#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quaternion,
    pub height: f32,
    pub width: f32,
    pub focal_length: f32,
    /// Near clip plane. Geometry nearer than this is cut away before the
    /// perspective divide. Too small and post-clip vertices project to
    /// coordinates large enough to exhaust f32 in the edge functions.
    pub z_near: f32,
}

impl Camera {
    pub fn new(
        position: Vec3,
        direction: Normal<Vec3>,
        height: f32,
        width: f32,
        focal_length: f32,
    ) -> Camera {
        let initial = Normal::new(Vec3::new(0.0, 0.0, -1.0));
        let rotation = rotation_between_vectors(&initial, &direction);

        Camera {
            position,
            rotation,
            height,
            width,
            focal_length,
            z_near: 0.1,
        }
    }

    pub fn basis(&self) -> Basis {
        Basis {
            right: Normal::new(self.rotation.transform(Vec3::new(1.0, 0.0, 0.0))),
            up: Normal::new(self.rotation.transform(Vec3::new(0.0, 1.0, 0.0))),
            forward: Normal::new(self.rotation.transform(Vec3::new(0.0, 0.0, -1.0))),
        }
    }

    pub fn to_view_with(&self, b: &Basis,vertex: Vec3) -> Vec3 {
        let v = vertex - self.position;

        Vec3::new(
            v.dot(*b.right),
            v.dot(*b.up),
            v.dot(*b.forward),
        )
    }

    pub fn projection(&self, screen: &Screen) -> Projection {
        Projection {
            scale: self.focal_length * screen.pixel_width as f32 / self.width,
            cx: screen.pixel_width as f32 * 0.5,
            cy: screen.pixel_height as f32 * 0.5,
        }
    }
}
