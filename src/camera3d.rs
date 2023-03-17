use crate::camera::*;
use crate::material::{Colour, Material};
use crate::mth::Vector2;
use crate::ray::{ray_trace, HitKind, RaySegment};
use crate::world_data::{Player, Region, World};
use sdl2::render::WindowCanvas;
use std::sync::mpsc;
use std::thread;
use crate::light_cache::LightingRegion;
use crate::lighting::{horizontal_surface_colour, vertical_surface_colour};

pub(crate) fn render(world: &World, window: &mut WindowCanvas, _delta_time: f64) {
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

fn render_column(world: &World, canvas: &mut RenderBuffer, raw_screen_x: usize) {
    // Adjust to what the x would be if the resolution factor was 1.
    // This makes lower resolutions have gaps instead of being squished on one side of the screen.
    let x = (raw_screen_x as f64 / RESOLUTION_FACTOR) as i32;

    let look_direction = ray_direction_for_x(x, &world.player.peek().look_direction);
    let segments = ray_trace(
        world.player.peek().pos,
        look_direction,
        &world.player.peek().region.peek(),
    );

    let mut cumulative_dist = 0.0;
    for segment in &segments {
        draw_floor_segment(canvas, segment, x, cumulative_dist);
        cumulative_dist += segment.line.length();
    }

    draw_wall_3d(
        &world.player.peek(),
        canvas,
        segments.last().unwrap(),
        look_direction,
        cumulative_dist,
        x,
    );
}

fn draw_floor_segment(
    canvas: &mut RenderBuffer,
    segment: &RaySegment,
    screen_x: i32,
    cumulative_dist: f64,
) {
    let ray_line = segment.line;

    let length = ray_line.length();
    let sample_length = 10.0;
    let sample_count = (length / sample_length).round() as i32 + 1;

    let samples = light_floor_segment(&segment, sample_length, sample_count + 1);

    // The top of the last floor segment is the bottom of this one.
    // The top of the floor segment is the bottom of where we'd draw if it was a wall.
    let (pixels_drawn, _) = project_to_screen(cumulative_dist);
    let mut last_top = SCREEN_HEIGHT - pixels_drawn;

    let steps_per_unit = 1.0;
    let units_per_step = 1.0 / steps_per_unit;
    let steps_per_sample = (sample_length.round() + 1.0) * steps_per_unit;

    for s in 0..sample_count {
        let current = samples[s as usize];
        let next = samples[(s + 1) as usize];

        for i in 0..(steps_per_sample as i32) {
            let dist = cumulative_dist + (s as f64 * sample_length) + (i as f64 * units_per_step);
            let (_, top) = project_to_screen(dist);
            let bottom = last_top;

            if (top - bottom).abs() < 2.0 {
                continue;
            }

            let t = i as f64 / steps_per_sample;
            let colour = current.lerp(&next, t); // TODO: its a quadratic not a line.
            canvas.set_draw_color(colour);
            canvas.draw_between(
                Vector2::of(screen_x as f64, bottom),
                Vector2::of(screen_x as f64, top),
            );

            last_top = top;
        }
    }
}

// the sample_count should be high enough that we have one past the end to lerp to
fn light_floor_segment(segment: &RaySegment, sample_length: f64, sample_count: i32) -> Vec<Colour> {
    let ray_line = segment.line;
    let mut samples: Vec<Colour> = Vec::with_capacity((sample_count) as usize);
    for i in 0..sample_count {
        let pos = ray_line.a.add(
            &ray_line
                .direction()
                .normalize()
                .scale(i as f64 * -sample_length),
        );
        samples.push(horizontal_surface_colour(segment.region, pos));
    }

    samples
}

fn draw_wall_3d(
    player: &Player,
    canvas: &mut RenderBuffer,
    hit: &RaySegment,
    ray_direction: Vector2,
    cumulative_dist: f64,
    screen_x: i32,
) {
    let hit_point = hit.line.b;
    let wall_normal = match &hit.kind {
        HitKind::HitNone { .. } => ray_direction,
        HitKind::HitWall { hit_wall, .. } => hit_wall.peek().normal,
        HitKind::HitPlayer { box_side, .. } => box_side.normal(),
    };

    let material = match &hit.kind {
        HitKind::HitNone { .. } => Material::new(0.0, 0.0, 0.0),
        HitKind::HitWall { hit_wall, .. } => hit_wall.peek().material,
        HitKind::HitPlayer { .. } => player.material,
    };

    let colour = vertical_surface_colour(
        hit.region,
        &hit_point,
        wall_normal,
        &material,
        ray_direction,
    );
    let (top, bottom) = project_to_screen(cumulative_dist);

    canvas.set_draw_color(colour);
    canvas.draw_between(
        Vector2::of(screen_x as f64, top),
        Vector2::of(screen_x as f64, bottom),
    );
}
