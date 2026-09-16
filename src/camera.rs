#![allow(dead_code)]

use crate::quaternion::{rotation_between_vectors, Quaternion};
use crate::screen::Screen;
use crate::vectors::{Normal, Vec2, Vec3};
#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quaternion,
    pub height: f32,
    pub width: f32,
    pub focal_length: f32,
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
        }
    }

    pub fn forward(&self) -> Normal<Vec3> {
        Normal::new(self.rotation.transform(Vec3::new(0.0, 0.0, -1.0)))
    }
    pub fn up(&self) -> Normal<Vec3> {
        Normal::new(self.rotation.transform(Vec3::new(0.0, 1.0, 0.0)))
    }
    pub fn right(&self) -> Normal<Vec3> {
        Normal::new(self.rotation.transform(Vec3::new(1.0, 0.0, 0.0)))
    }
    
    pub fn to_view(&self, vertex: Vec3) -> Vec3 {
        let v = vertex - self.position;
        Vec3::new(v.dot(*self.right()), v.dot(*self.up()), v.dot(*self.forward()))
    }

    pub fn project_view(&self, v: Vec3, screen: &Screen) -> (Vec2, f32) {
        let x = v.x / v.z * self.focal_length;
        let y = v.y / v.z * self.focal_length;

        let aspect = screen.pixel_width as f32 / screen.pixel_height as f32;
        let height = self.width / aspect;

        let px = (x / self.width + 0.5) * screen.pixel_width as f32;
        let py = (0.5 - y / height) * screen.pixel_height as f32;

        (Vec2::new(px, py), v.z)
    }
}
