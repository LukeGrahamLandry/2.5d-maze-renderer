use crate::camera::*;
use crate::mth::Vector2;
use crate::ray::RaySegment;
use crate::world::{Region, World};

pub(crate) fn render<R: RenderStrategy>(world: &World, window: &mut R) {
    for x in 0..((SCREEN_WIDTH as f64 * RESOLUTION_FACTOR) as i32) as usize {
        render_column(world, window, x);
    }
}

fn render_column<R: RenderStrategy>(world: &World, canvas: &mut R, raw_screen_x: usize) {
    // Adjust to what the x would be if the resolution factor was 1.
    // This makes lower resolutions have gaps instead of being squished on one side of the screen.
    let x = (raw_screen_x as f64 / RESOLUTION_FACTOR) as i32;

    let look_direction = ray_direction_for_x(x, &world.player().look_direction);
    let region = world.get_region(world.player().entity.region);
    let segments = world.ray_trace(region.id, world.player().entity.pos, look_direction);

    let mut cumulative_dist = 0.0;
    for segment in &segments {
        draw_floor_segment(
            canvas,
            world.get_region(segment.region),
            segment,
            x,
            cumulative_dist,
        );
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

fn draw_floor_segment<R: RenderStrategy>(
    canvas: &mut R,
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
    let sample_count = (length / sample_length).ceil() as i32 + 1;

    // The top of the last floor segment is the bottom of this one.
    // The top of the floor segment is the bottom of where we'd draw if it was a wall.
    let (pixels_drawn, _) = project_to_screen(cumulative_dist);
    let mut last_top = SCREEN_HEIGHT - pixels_drawn;

    for i in 0..sample_count {
        // the -1 fixes the square of black at the base of a wall. It uses the colour right before the wall instead of right after the wall (which would be in shadow)
        let pos = ray_line
            .a
            .add(&ray_direction.scale((i - 2) as f64 * -sample_length));
        let colour = region.horizontal_surface_colour_memoized(pos);

        let dist = cumulative_dist + (i as f64 * sample_length);
        let (_, top) = project_to_screen(dist);
        let bottom = last_top;

        // if (top - bottom).abs() < 2.0 {
        //     continue;
        // }

        canvas.set_draw_color(colour);
        canvas.draw_between(
            Vector2::of(screen_x as f64, bottom),
            Vector2::of(screen_x as f64, top),
        );

        last_top = top;
    }
}

fn draw_wall_3d<R: RenderStrategy>(
    canvas: &mut R,
    region: &Region,
    hit: &RaySegment,
    ray_direction: Vector2,
    cumulative_dist: f64,
    screen_x: i32,
) {
    assert_eq!(region.id, hit.region);
    match hit.hit_wall {
        None => {}
        Some(wall) => {
            let colour = region.vertical_surface_colour(
                &hit.line.get_b(),
                wall,
                hit.line.direction().negate(),
            );
            let (top, bottom) = project_to_screen(cumulative_dist);

            canvas.set_draw_color(colour);
            canvas.draw_between(
                Vector2::of(screen_x as f64, top),
                Vector2::of(screen_x as f64, bottom),
            );
        }
    }
}
