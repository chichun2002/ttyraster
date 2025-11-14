use crate::vectors::Vec2;

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
        Pixel { r: 0, g: 0, b: 0 }
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
}

impl Screen {
    pub fn new(pixel_width: u32, pixel_height: u32) -> Self {
        Screen {
            pixel_width,
            pixel_height,
            pixels: vec![Pixel::black(); (pixel_width * pixel_height) as usize].into_boxed_slice(),
        }
    }

    pub fn clear(&mut self) {
        for pixel in self.pixels.iter_mut() {
            *pixel = Pixel::black();
        }
    }

    pub fn triangle_bounds(&mut self, v0: Vec2, v1: Vec2, v2: Vec2) {
        let x_max = v0.x.max(v1.x).max(v2.x);
        let y_max = v0.y.max(v1.y).max(v2.y);
        let x_min = v0.x.min(v1.x).min(v2.x);
        let y_min = v0.y.min(v1.y).min(v2.y);

        self.draw_pixel(Vec2::new(x_min, y_min), Pixel::new(150, 150, 150));
        self.draw_pixel(Vec2::new(x_max, y_max), Pixel::new(150, 150, 150));
    }

    pub fn draw_triangle(&mut self, v0: Vec2, v1: Vec2, v2: Vec2) {
        self.draw_pixel(v0, Pixel::white());
        self.draw_pixel(v1, Pixel::white());
        self.draw_pixel(v2, Pixel::white());

        self.triangle_bounds(v0, v1, v2);
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
        use crossterm::{cursor, execute};
        use std::io::{stdout, Write};

        for y in 0..self.pixel_height {
            execute!(stdout(), cursor::MoveTo(0, y as u16))?;
            for x in 0..self.pixel_width {
                let index = (y * self.pixel_width + x) as usize;
                let pixel = self.pixels[index];

                let char = if pixel.r > 200 && pixel.g > 200 && pixel.b > 200 {
                    '█' // White pixel
                } else if pixel.r > 100 || pixel.g > 100 || pixel.b > 100 {
                    '░' // Gray pixel
                } else {
                    ' ' // Black pixel
                };
                print!("{}", char);
            }
        }
        stdout().flush()?;
        Ok(())
    }
}
