mod camera;
mod object;
mod quaternion;
mod ray;
mod screen;
mod triangle;
mod vectors;

use crate::camera::Camera;
use crate::quaternion::from_axis_angle;
use crate::screen::Screen;
use crate::vectors::{Normal, Vec3};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use std::io::{stdout, Write};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let triangle = Triangle::new(
    //     Vec3::new(5.0, 0.2, 0.0),
    //     Vec3::new(1.0, 0.8, 0.0),
    //     Vec3::new(3.0, 5.2, 0.0),
    // );
    let mut object = object::load_model(Path::new(
        "/home/tristan/Documents/software_rasterizer/models/bunny.obj",
    ))?;

    let mut camera = Camera::new(
        Vec3::new(0.0, 1.0, 5.0),
        Normal::new(Vec3::new(0.0, 0.0, -1.0)),
        1.0,
        1.0,
        0.2,
    );

    enable_raw_mode()?;
    execute!(stdout(), cursor::Hide)?;

    let move_speed = 0.5;
    let rotate_speed = 0.1;
    let focal_speed = 0.05;

    loop {
        // Move cursor to top-left
        execute!(stdout(), cursor::MoveTo(0, 0))?;

        // Get terminal size and create screen
        let (term_width, term_height) = size()?;
        let screen_height = (term_height - 8).min(50) as u32; // Leave room for controls
        let screen_width = term_width.min(1000) as u32;

        let mut screen = Screen::new(screen_width, screen_height);

        let rotation = from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.02);
        object.rotation = rotation * object.rotation;

        // Project vertices and draw triangle
        let draw_start = Instant::now();
        // let p1 = camera.project(triangle.p1, &screen);
        // let p2 = camera.project(triangle.p2, &screen);
        // let p3 = camera.project(triangle.p3, &screen);
        for f in object.faces.iter() {
            let v0 = object.rotation.transform(object.vertices[f[0]]);
            let v1 = object.rotation.transform(object.vertices[f[1]]);
            let v2 = object.rotation.transform(object.vertices[f[2]]);

            let p1 = camera.project(v0, &screen);
            let p2 = camera.project(v1, &screen);
            let p3 = camera.project(v2, &screen);
            screen.draw_triangle(p1, p2, p3);
        }
        let draw_time = draw_start.elapsed();

        // screen.draw_triangle(p1, p2, p3);

        let render_start = Instant::now();
        screen.render()?;
        let render_time = render_start.elapsed();

        // Show controls and performance stats at bottom
        execute!(stdout(), cursor::MoveTo(0, screen_height as u16 + 1))?;
        print!(
            "Controls: W/A/S/D=Move | Q/E=Rotate | M/N=Focal | ESC=Exit | Draw: {:.2}ms | Render: {:.2}ms | Camera: pos={:?}, focal={:.2}",
            draw_time.as_millis(),
            render_time.as_millis(),
            camera.position,
            camera.focal_length
        );
        stdout().flush()?;

        // Poll for input (non-blocking) - use 0ms for max speed, or set to 16ms for ~60 FPS cap
        if event::poll(std::time::Duration::from_millis(0))? {
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
    }

    execute!(stdout(), cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}
