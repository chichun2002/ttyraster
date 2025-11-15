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
                    fn parse_vertex(s: &str) -> Result<usize> {
                        let vertex_str = s.split('/').next().context("Empty vertex reference")?;
                        let idx: usize = vertex_str.parse().context("Invalid vertex index")?;
                        Ok(idx - 1)
                    }
                    let idx1 = parse_vertex(v1)?;
                    let idx2 = parse_vertex(v2)?;
                    let idx3 = parse_vertex(v3)?;

                    faces.push([idx1, idx2, idx3]);

                    for i in 3..parts.len() - 1 {
                        let idx_next = parse_vertex(parts[i + 1])?;
                        faces.push([idx1, parse_vertex(parts[i])?, idx_next]);
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
