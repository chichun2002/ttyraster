mod camera;
mod quaternion;
mod ray;
mod screen;
mod triangle;
mod vectors;

use crate::camera::Camera;
use crate::quaternion::from_axis_angle;
use crate::screen::Screen;
use crate::triangle::Triangle;
use crate::vectors::{Normal, Vec3};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use std::io::{stdout, Write};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let triangle = Triangle::new(
        Vec3::new(5.0, 0.2, 0.0),
        Vec3::new(1.0, 0.8, 0.0),
        Vec3::new(3.0, 5.2, 0.0),
    );

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Normal::new(Vec3::new(0.0, 0.0, -1.0)),
        1.0,
        1.0,
        0.2,
    );

    enable_raw_mode()?;

    let move_speed = 0.5;
    let rotate_speed = 0.1;
    let focal_speed = 0.05;

    loop {
        // Clear screen and move cursor to top-left
        execute!(stdout(), cursor::MoveTo(0, 0), Clear(ClearType::All))?;

        // Get terminal size and create screen
        let (term_width, term_height) = size()?;
        let screen_height = (term_height - 8).min(50) as u32;  // Leave room for controls
        let screen_width = term_width.min(100) as u32;

        let mut screen = Screen::new(screen_width, screen_height);

        // Project vertices and draw triangle
        let proj_start = Instant::now();
        let p1 = camera.project(triangle.p1, &screen);
        let p2 = camera.project(triangle.p2, &screen);
        let p3 = camera.project(triangle.p3, &screen);
        let proj_time = proj_start.elapsed();

        screen.draw_triangle(p1, p2, p3);

        let render_start = Instant::now();
        screen.render()?;
        let render_time = render_start.elapsed();

        // Show controls and performance stats at bottom
        execute!(stdout(), cursor::MoveTo(0, screen_height as u16 + 1))?;
        print!("Controls: W/A/S/D=Move | Q/E=Rotate | M/N=Focal | ESC=Exit | Proj: {:.2}µs | Render: {:.2}ms | Camera: pos={:?}, focal={:.2}",
            proj_time.as_micros(), render_time.as_secs_f64() * 1000.0, camera.position, camera.focal_length);
        stdout().flush()?;

        // Wait for input
        if let Event::Key(key_event) = event::read()? {
            let forward = *camera.forward();
            let right = *camera.right();

            match key_event.code {
                KeyCode::Char('w') => {
                    camera.position = camera.position + forward * move_speed;
                }
                KeyCode::Char('s') => {
                    camera.position = camera.position - forward * move_speed;
                }
                KeyCode::Char('a') => {
                    camera.position = camera.position - right * move_speed;
                }
                KeyCode::Char('d') => {
                    camera.position = camera.position + right * move_speed;
                }
                KeyCode::Char('q') => {
                    // Rotate left around Y axis
                    let rotation = from_axis_angle(Vec3::new(0.0, 1.0, 0.0), rotate_speed);
                    camera.rotation = rotation * camera.rotation;
                }
                KeyCode::Char('e') => {
                    // Rotate right around Y axis
                    let rotation = from_axis_angle(Vec3::new(0.0, 1.0, 0.0), -rotate_speed);
                    camera.rotation = rotation * camera.rotation;
                }
                KeyCode::Char('m') => {
                    camera.focal_length += focal_speed;
                }
                KeyCode::Char('n') => {
                    camera.focal_length = (camera.focal_length - focal_speed).max(0.01);
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}
