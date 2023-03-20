use crate::camera::*;
use crate::material::{Colour};
use crate::mth::Vector2;
use crate::ray::{RaySegment};
use sdl2::render::WindowCanvas;
use std::sync::mpsc;
use std::thread;
use crate::world::{Region, World};

pub(crate) fn render(world: & World , window: &mut WindowCanvas, _delta_time: f64) {
    let (sender, receiver) = mpsc::channel();

    let thread_count = 3 as usize;
    thread::scope(|s| {
        {
            let sender = sender;
            for i in 0..thread_count {
                let sender = sender.clone();
                s.spawn(move || {
                    let mut line_chunk_size = 500;
                    let mut lines = Vec::with_capacity(line_chunk_size);

                    for x in (i..((SCREEN_WIDTH as f64 * RESOLUTION_FACTOR) as i32) as usize).step_by(thread_count) {
                        let mut handler = |line| {
                            lines.push(line);
                        };

                        let mut canvas = RenderBuffer::new(&mut handler);
                        render_column(world, &mut canvas, x);

                        let size = lines.len();
                        if size > line_chunk_size {
                            sender.send(lines).expect("Failed to send lines.");
                            lines = Vec::with_capacity(size);
                        }
                    }

                    sender.send(lines).expect("Failed to send lines.");
                });
            }
        }

        // most lines are really short. from the floor. this is dumb.
        // lights should memoize the colour value at a certain radius on a given frame.
        // do the sampling in world space and draw fixed lengths lerping between them.

        for lines in receiver {
            draw_lines(window, lines);
        }
    });
}

fn render_column(world: &World , canvas: &mut RenderBuffer, raw_screen_x: usize) {
    // Adjust to what the x would be if the resolution factor was 1.
    // This makes lower resolutions have gaps instead of being squished on one side of the screen.
    let x = (raw_screen_x as f64 / RESOLUTION_FACTOR) as i32;

    let look_direction = ray_direction_for_x(x, &world.player().look_direction);
    let region = world.get_region(world.player().entity.region);
    let segments = world.ray_trace(
        region.id,
        world.player().entity.pos,
        look_direction,
    );

    let mut cumulative_dist = 0.0;
    for segment in &segments {
        draw_floor_segment(canvas, world.get_region(segment.region), segment, x, cumulative_dist);
        cumulative_dist += segment.line.length();
    }

    let segment = segments.last().unwrap();
    draw_wall_3d(
        canvas,
        world.get_region(segment.region),
        segment,
        look_direction,
        cumulative_dist,
        x,
    );
}

fn draw_floor_segment(
    canvas: &mut RenderBuffer,
    region: &Region,
    segment: &RaySegment,
    screen_x: i32,
    cumulative_dist: f64,
) {
    assert_eq!(region.id, segment.region);

    let ray_line = segment.line;
    let ray_direction = ray_line.direction().normalize();

    let length = ray_line.length();
    let sample_length = 1.0;
    let sample_count = (length / sample_length).round() as i32 + 1;

    // The top of the last floor segment is the bottom of this one.
    // The top of the floor segment is the bottom of where we'd draw if it was a wall.
    let (pixels_drawn, _) = project_to_screen(cumulative_dist);
    let mut last_top = SCREEN_HEIGHT - pixels_drawn;

    for i in 0..sample_count {
        let pos = ray_line.a.add(&ray_direction.scale(i as f64 * -sample_length));
        let colour = region.horizontal_surface_colour_memoized(pos);

        let dist = cumulative_dist + (i as f64 * sample_length);
        let (_, top) = project_to_screen(dist);
        let bottom = last_top;

        if (top - bottom).abs() < 2.0 {
            continue;
        }

        canvas.set_draw_color(colour);
        canvas.draw_between(
            Vector2::of(screen_x as f64, bottom),
            Vector2::of(screen_x as f64, top),
        );

        last_top = top;
    }
}



fn draw_wall_3d(canvas: &mut RenderBuffer, region: &Region, hit: &RaySegment, ray_direction: Vector2, cumulative_dist: f64, screen_x: i32) {
    assert_eq!(region.id, hit.region);
    match hit.hit_wall {
        None => {}
        Some(wall) => {
            let colour = region.vertical_surface_colour(&hit.line.get_b(), wall, hit.line.direction().negate());
            let (top, bottom) = project_to_screen(cumulative_dist);

            canvas.set_draw_color(colour);
            canvas.draw_between(
                Vector2::of(screen_x as f64, top),
                Vector2::of(screen_x as f64, bottom),
            );
        }
    }
}
