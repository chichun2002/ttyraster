use crate::quaternion::Quaternion;
use crate::vectors::Vec3;
use anyhow::{Context, Result};
use std::fs::read;
use std::path::Path;

pub trait ModelLoader {
    fn load(data: &[u8]) -> Result<Object>;
    fn supports_extension(ext: &str) -> bool;
}

pub struct Object {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<[usize; 3]>,
    pub rotation: Quaternion,
}

pub struct ObjLoader;
impl ModelLoader for ObjLoader {
    fn load(data: &[u8]) -> Result<Object> {
        let text = std::str::from_utf8(data).context("Failed to parse file as utf-8")?;

        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for line in text.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();

            match parts.as_slice() {
                ["v", x, y, z] => {
                    vertices.push(Vec3::new(
                        x.parse().context("Invalid vertex x")?,
                        y.parse().context("Invalid vertex y")?,
                        z.parse().context("Invalid vertex z")?,
                    ));
                }
                ["f", v1, v2, v3, ..] => {
                    fn parse_vertex(s: &str, vertex_count: usize) -> Result<usize> {
                        let vertex_str = s.split('/').next().context("Empty vertex reference")?;
                        let idx: isize = vertex_str.parse().context("Invalid vertex index")?;

                        // Handle negative indices (relative to current position)
                        let absolute_idx = if idx < 0 {
                            (vertex_count as isize + idx) as usize
                        } else {
                            (idx - 1) as usize // OBJ indices are 1-based
                        };

                        Ok(absolute_idx)
                    }
                    let idx1 = parse_vertex(v1, vertices.len())?;
                    let idx2 = parse_vertex(v2, vertices.len())?;
                    let idx3 = parse_vertex(v3, vertices.len())?;

                    faces.push([idx1, idx2, idx3]);

                    for i in 3..parts.len() - 1 {
                        let idx_next = parse_vertex(parts[i + 1], vertices.len())?;
                        faces.push([idx1, parse_vertex(parts[i], vertices.len())?, idx_next]);
                    }
                }
                _ => {}
            }
        }
        Ok(Object {
            vertices,
            faces,
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        })
    }
    fn supports_extension(ext: &str) -> bool {
        matches!(ext, "obj")
    }
}

pub fn load_model(path: &Path) -> Result<Object> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .context("Failed to get extension of file")?;

    let data = read(path).context("Failed to read file")?;

    if ObjLoader::supports_extension(ext) {
        return ObjLoader::load(&data);
    }
    anyhow::bail!("Unsupported file type {}", ext);
}
