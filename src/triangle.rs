#![allow(dead_code)]

use crate::vectors::Vec3;

#[derive(Debug)]
pub struct Triangle {
    pub p1: Vec3,
    pub p2: Vec3,
    pub p3: Vec3,
}

impl std::ops::Index<usize> for Triangle {
    type Output = Vec3;

    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.p1,
            1 => &self.p2,
            2 => &self.p3,
            _ => panic!("Index out of bounds for Triangle"),
        }
    }
}

impl Triangle {
    pub fn new(p1: Vec3, p2: Vec3, p3: Vec3) -> Triangle {
        Triangle { p1, p2, p3 }
    }

    pub fn map(self, mut f: impl FnMut(Vec3) -> Vec3) -> Triangle {
        Triangle {
            p1: f(self.p1),
            p2: f(self.p2),
            p3: f(self.p3),
        }
    }

    pub fn clip_z(&self, z_near: f32) -> Vec<Triangle> {
        let mut poly: Vec<Vec3> = vec![];

        for i in 0..3 {
            let a = self[i];
            let b = self[(i + 1) % 3];

            if a.z >= z_near {
                poly.push(a);
            }
            if (a.z >= z_near) != (b.z >= z_near) {
                let t = (z_near - a.z) / (b.z - a.z);
                poly.push(a + (b - a) * t);
            }
        }

        match poly.len() {
            3 => vec![Triangle::new(poly[0], poly[1], poly[2])],
            4 => vec![
                Triangle::new(poly[0], poly[1], poly[2]),
                Triangle::new(poly[0], poly[2], poly[3]),
            ],
            _ => vec![],
        }
    }
}