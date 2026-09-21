#![allow(dead_code)]

use crate::{helper, vectors::Vec2};

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Pixel {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Pixel { r, g, b }
    }
    pub fn black() -> Self {
        Pixel { r: 1, g: 1, b: 1 } // Use near-black to avoid terminal transparency
    }

    pub fn white() -> Self {
        Pixel {
            r: 255,
            g: 255,
            b: 255,
        }
    }
}

pub struct Screen {
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub pixels: Box<[Pixel]>,
    pub depth: Box<[f32]>,
}

impl Screen {
    pub fn new(pixel_width: u32, pixel_height: u32) -> Self {
        Screen {
            pixel_width,
            pixel_height,
            pixels: vec![Pixel::black(); (pixel_width * pixel_height) as usize].into_boxed_slice(),
            depth: vec![f32::MAX; (pixel_width * pixel_height) as usize].into_boxed_slice(),
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(Pixel::black());
        self.depth.fill(f32::MAX);
    }

    pub fn triangle_bounds(&mut self, v0: Vec2, v1: Vec2, v2: Vec2) -> (Vec2, Vec2) {
        let x_max = v0.x.max(v1.x).max(v2.x);
        let y_max = v0.y.max(v1.y).max(v2.y);
        let x_min = v0.x.min(v1.x).min(v2.x);
        let y_min = v0.y.min(v1.y).min(v2.y);

        // Debug: Draw triangle bounds
        // self.draw_pixel(Vec2::new(x_min, y_min), Pixel::new(150, 150, 150));
        // self.draw_pixel(Vec2::new(x_max, y_max), Pixel::new(150, 150, 150));

        (Vec2::new(x_min, y_min), Vec2::new(x_max, y_max))
    }

    pub fn draw_triangle(&mut self, v0: (Vec2, f32), v1: (Vec2, f32), v2: (Vec2, f32)) {

        fn edge_function(a: Vec2, b: Vec2, c: Vec2) -> f32 {
            (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
        }

        let bounds = self.triangle_bounds(v0.0, v1.0, v2.0);

        let w = self.pixel_width as f32;
        let h = self.pixel_height as f32;

        let x0 = bounds.0.x.floor().clamp(0.0, w) as usize;
        let x1 = bounds.1.x.ceil().clamp(0.0, w) as usize;
        let y0 = bounds.0.y.floor().clamp(0.0, h) as usize;
        let y1 = bounds.1.y.ceil().clamp(0.0, h) as usize;

        // 1/z is linear in screen space, z is not. Interpolate the
        // reciprocals and flip back per pixel.
        let iz0 = 1.0 / v0.1;
        let iz1 = 1.0 / v1.1;
        let iz2 = 1.0 / v2.1;

        for y in y0..y1 {
            for x in x0..x1 {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);

                let w0 = edge_function(v0.0, v1.0, p);
                let w1 = edge_function(v1.0, v2.0, p);
                let w2 = edge_function(v2.0, v0.0, p);

                let inside =
                    (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);

                if inside {
                    let depth = (w0 + w1 + w2) / (w0 * iz2 + w1 * iz0 + w2 * iz1);
                    let idx = y * w as usize + x;
                    if depth < self.depth[idx] {
                        self.depth[idx] = depth;
                        // self.pixels[idx] = depth_map(depth)
                    }
                }
            }
        }
    }

    pub fn pixel_transform(&mut self) {
        fn depth_map(depth: f32, max_depth: f32, min_depth: f32) -> Pixel {
            let intensity = helper::lerp(depth, min_depth, max_depth, 255.0, 0.0) as u8;
            Pixel::new(intensity, intensity, intensity)
        }

        let max_depth = self.depth.iter().copied().filter(|&x| x != f32::MAX && !x.is_nan()).max_by(f32::total_cmp);
        let min_depth = self.depth.iter().copied().filter(|&x| x != f32::MAX && !x.is_nan()).min_by(f32::total_cmp);

        if let (Some(min), Some(max)) = (min_depth, max_depth) {
            for (i, &depth) in self.depth.iter().enumerate() {
                if depth != f32::MAX && !depth.is_nan() {
                    self.pixels[i] = depth_map(depth, max, min);
                }
            }
        }
    }

    pub fn draw_pixel(&mut self, pos: Vec2, color: Pixel) {
        // Check if coordinates are valid (positive and within bounds)
        if pos.x >= 0.0 && pos.y >= 0.0 {
            let x = pos.x as u32;
            let y = pos.y as u32;

            if x < self.pixel_width && y < self.pixel_height {
                let index = (y * self.pixel_width + x) as usize;
                self.pixels[index] = color;
            }
        }
    }

    pub fn render(&self) -> std::io::Result<()> {
        use std::io::{stdout, Write};
        use std::fmt::Write as FmtWrite;

        // Build entire frame in a string buffer with ANSI escape codes for positioning
        let mut buffer = String::with_capacity(
            (self.pixel_width * self.pixel_height + self.pixel_height * 10) as usize,
        );

        for y in (0..self.pixel_height).step_by(2) {
            // Add cursor positioning for each line (y/2 because we process 2 rows per terminal line)
            write!(&mut buffer, "\x1b[{};{}H", y / 2 + 1, 1).unwrap();

            for x in 0..self.pixel_width {
                let pixel_top = self.pixels[(y * self.pixel_width + x) as usize];
                let pixel_bottom = if y + 1 < self.pixel_height {
                    self.pixels[((y + 1) * self.pixel_width + x) as usize]
                } else {
                    pixel_top
                };

                buffer.push_str(&format!(
                    "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m▄",
                    pixel_top.r,
                    pixel_top.g,
                    pixel_top.b,
                    pixel_bottom.r,
                    pixel_bottom.g,
                    pixel_bottom.b
                ));

                // let char = if pixel.r > 200 && pixel.g > 200 && pixel.b > 200 {
                //     '█' // White pixel
                // } else if pixel.r > 100 || pixel.g > 100 || pixel.b > 100 {
                //     '░' // Gray pixel
                // } else {
                //     ' ' // Black pixel
                // };
                // buffer.push(char);
            }
            buffer.push_str("\x1b[0m")
        }

        // Write entire frame at once
        print!("{}", buffer);
        stdout().flush()?;
        Ok(())
    }
}
