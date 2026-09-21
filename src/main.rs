#![allow(dead_code)]

mod camera;
mod object;
mod quaternion;
mod ray;
mod screen;
mod triangle;
mod vectors;
mod helper;

use crate::camera::{Camera, Projection};
use crate::quaternion::from_axis_angle;
use crate::screen::Screen;
use crate::triangle::Triangle;
use crate::vectors::{Normal, Vec2, Vec3};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use std::io::{stdout, Write};
use std::path::Path;
use std::time::Instant;

/// Project and rasterize one view-space triangle. Only needed on the clipping
/// path; the common case reads pre-projected vertices straight out of the cache.
fn draw(a: Vec3, b: Vec3, c: Vec3, projection: &Projection, screen: &mut Screen) {
    screen.draw_triangle(
        projection.project(a),
        projection.project(b),
        projection.project(c),
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let triangle = Triangle::new(
    //     Vec3::new(5.0, 0.2, 0.0),
    //     Vec3::new(1.0, 0.8, 0.0),
    //     Vec3::new(3.0, 5.2, 0.0),
    // );
    let mut object = object::load_model(Path::new(
        "/home/tristan/Documents/software_rasterizer/models/dragon.obj",
    ))?;

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 2.0),
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

    // Everything below is reused across frames so the hot loop never allocates.
    let (mut term_width, mut term_height) = size()?;
    // Double height since we use half-blocks (2 pixels per terminal line),
    // leaving room for the controls line.
    let mut screen = Screen::new(term_width as u32, ((term_height - 3) * 2) as u32);
    let mut view_vertices: Vec<Vec3> = Vec::with_capacity(object.vertices.len());
    let mut screen_vertices: Vec<(Vec2, f32)> = Vec::with_capacity(object.vertices.len());

    // Exponential moving averages, because raw per-frame timings are unreadable.
    // 0.05 gives a time constant of roughly 20 frames.
    const SMOOTHING: f64 = 0.05;
    let mut draw_avg: Option<f64> = None;
    let mut render_avg: Option<f64> = None;
    // Noise only ever makes a frame slower, so the fastest frame seen is the
    // cleanest estimate of what the work actually costs.
    let mut draw_min = f64::INFINITY;

    loop {
        // Move cursor to top-left
        execute!(stdout(), cursor::MoveTo(0, 0))?;

        let (tw, th) = size()?;
        if (tw, th) != (term_width, term_height) {
            term_width = tw;
            term_height = th;
            screen = Screen::new(tw as u32, ((th - 3) * 2) as u32);
        } else {
            screen.clear();
        }
        let screen_height = screen.pixel_height;

        let rotation = from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.02);
        object.rotation = (rotation * object.rotation).normalized();

        // Project vertices and draw triangle
        let draw_start = Instant::now();

        let basis = camera.basis();
        let projection = camera.projection(&screen);
        let z_near = camera.z_near;

        // Each vertex is shared by about six faces, so transform and project
        // once here rather than once per face that references it.
        view_vertices.clear();
        view_vertices.extend(
            object
                .vertices
                .iter()
                .map(|&v| camera.to_view_with(&basis, object.rotation.transform(v))),
        );

        screen_vertices.clear();
        screen_vertices.extend(view_vertices.iter().map(|&v| projection.project(v)));

        for f in object.faces.iter() {
            let (a, b, c) = (view_vertices[f[0]], view_vertices[f[1]], view_vertices[f[2]]);

            if a.z >= z_near && b.z >= z_near && c.z >= z_near {
                // Common case: nothing to clip, vertices already projected.
                screen.draw_triangle(
                    screen_vertices[f[0]],
                    screen_vertices[f[1]],
                    screen_vertices[f[2]],
                );
            } else if a.z < z_near && b.z < z_near && c.z < z_near {
                continue;
            } else {
                for t in Triangle::new(a, b, c).clip_z(z_near) {
                    draw(t.p1, t.p2, t.p3, &projection, &mut screen);
                }
            }
        }

        screen.pixel_transform();

        let draw_time = draw_start.elapsed();

        let render_start = Instant::now();
        screen.render()?;
        let render_time = render_start.elapsed();

        // Show controls and performance stats at bottom
        fn smooth(avg: &mut Option<f64>, sample: f64) -> f64 {
            let next = match *avg {
                Some(a) => a + (sample - a) * SMOOTHING,
                None => sample,
            };
            *avg = Some(next);
            next
        }
        let draw_sample = draw_time.as_secs_f64() * 1000.0;
        let draw_ms = smooth(&mut draw_avg, draw_sample);
        let render_ms = smooth(&mut render_avg, render_time.as_secs_f64() * 1000.0);
        draw_min = draw_min.min(draw_sample);

        execute!(stdout(), cursor::MoveTo(0, screen_height as u16 / 2 + 1))?;
        print!(
            "W/A/S/D=Move | Q/E=Rotate | M/N=Focal | R=Reset min | ESC=Exit | Draw: {:6.2}ms (min {:6.2}) | Render: {:6.2}ms | focal={:.2}",
            draw_ms,
            draw_min,
            render_ms,
            camera.focal_length
        );
        stdout().flush()?;

        // Poll for input (non-blocking) - use 0ms for max speed, or set to 16ms for ~60 FPS cap
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key_event) = event::read()? {
                let forward = *basis.forward;
                let right = *basis.right;

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
                        camera.rotation = (rotation * camera.rotation).normalized();
                    }
                    KeyCode::Char('e') => {
                        // Rotate right around Y axis
                        let rotation = from_axis_angle(Vec3::new(0.0, 1.0, 0.0), -rotate_speed);
                        camera.rotation = (rotation * camera.rotation).normalized();
                    }
                    KeyCode::Char('m') => {
                        camera.focal_length += focal_speed;
                    }
                    KeyCode::Char('n') => {
                        camera.focal_length = (camera.focal_length - focal_speed).max(0.01);
                    }
                    KeyCode::Char('r') => {
                        draw_min = f64::INFINITY;
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
