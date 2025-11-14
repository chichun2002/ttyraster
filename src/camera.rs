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
    pub fn project(&self, vertex: Vec3, screen: &Screen) -> Vec2 {
        let vertex = vertex - self.position;
        let x = vertex.dot(*self.right()) / vertex.dot(*self.forward()) * self.focal_length;
        let y = vertex.dot(*self.up()) / vertex.dot(*self.forward()) * self.focal_length;

        let aspect_ratio = screen.pixel_width as f32 / screen.pixel_height as f32;
        let height = self.width / aspect_ratio;

        let pixel_x = (x / self.width + 0.5) * screen.pixel_width as f32;
        let pixel_y = (0.5 - y / height) * screen.pixel_height as f32;

        Vec2::new(pixel_x, pixel_y)
    }
}
