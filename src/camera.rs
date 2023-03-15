use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::{Rc, Weak};
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::player::Player;
use crate::ray::{HitKind, HitResult, ray_trace, single_ray_trace, trace_clear_portal_light};

use crate::world::{Region, Wall, World};

const FOV_DEG: i32 = 45;
const SCREEN_HEIGHT: f64 = 600.0;
pub const SCREEN_WIDTH: u32 = 800;
const RESOLUTION_FACTOR: f64 = 1.0;


fn draw_between(canvas: &mut WindowCanvas, start: Vector2, end: Vector2){
    canvas.draw_line(start.sdl(), end.sdl()).unwrap();
}

fn draw_line(canvas: &mut WindowCanvas, line: &LineSegment2){
    draw_between(canvas, line.a, line.b);
}

pub(crate) fn render2d(world: &World, canvas: &mut WindowCanvas, _delta_time: f64){
    let half_player_size = 5;
    let offset = world.player.borrow().pos.subtract(&Vector2::of((SCREEN_WIDTH / 2) as f64, SCREEN_HEIGHT / 2.0));

    // Draw the regions.
    for region in world.regions.iter() {
        // Draw lights
        let ray_count = 128;
        for light in &region.borrow().lights {
            let hit_colour = light.intensity.scale(0.3);
            let miss_colour = light.intensity.scale(0.1);
            for r in 0..ray_count {
                // Draw rays
                let direction = Vector2::from_angle(r as f64 * PI / (ray_count as f64 / 2.0), 1.0);
                let ray_start = light.pos.add(&direction.scale(3.0));
                let segment = single_ray_trace(ray_start, direction, &region);

                draw_ray_segment_2d(canvas, offset, &segment, hit_colour, miss_colour);
            }
        }

        // Draw walls
        for wall in region.borrow().walls.iter() {
            let contains_player = Rc::ptr_eq(&world.player.borrow().region, region);
            draw_wall_2d(canvas, offset, &wall.borrow(), contains_player);

            // Draw saved lights
            let wall = wall.borrow();
            for (light, fake_location) in &wall.lights {
                let light_fake_origin = fake_location.get_b();
                let light_hits_portal_at = fake_location.get_a();

                canvas.set_draw_color(light.intensity.scale(0.2).sdl());
                for r in 0..ray_count {
                    let direction = Vector2::from_angle(r as f64 * PI / (ray_count as f64 / 2.0), 1.0);

                    let light_toward_portal = LineSegment2::from(light_fake_origin, direction.scale(100.0));
                    let point_on_portal = wall.line.algebraic_intersection(&light_toward_portal);
                    let actually_crosses_portal = wall.line.contains(&point_on_portal);
                    if !actually_crosses_portal {
                        continue;
                    }

                    let direction = point_on_portal.subtract(&light_fake_origin);
                    let segments = ray_trace(point_on_portal.add(&direction.tiny()), direction, &region);
                    let wall_hit_point = segments.last().unwrap().line.b;

                    let line = trace_clear_portal_light(*fake_location, wall.line, wall_hit_point, region);
                    match line {
                        None => {}
                        Some(line) => {
                            draw_between(canvas, line.a.subtract(&offset), line.b.subtract(&offset));
                        }
                    }
                }

                canvas.set_draw_color(light.intensity.scale(1.0).sdl());
                draw_between(canvas, fake_location.a.subtract(&offset), fake_location.b.subtract(&offset));
            }
        }
    }

    // Draw view rays.
    for x in 0..(SCREEN_WIDTH as i32) {
        if x % 15 != 0 {
            continue;
        }

        let look_direction = ray_direction_for_x(x, &world.player.borrow().look_direction);
        let segments = ray_trace(world.player.borrow().pos, look_direction, &world.player.borrow().region);
        let hit_colour = Colour::rgb(150, 150, 0);
        let miss_colour = Colour::rgb(150, 150, 150);
        for segment in &segments {
            draw_ray_segment_2d(canvas, offset, segment, hit_colour, miss_colour);
        }
    }

    // Draw the player.
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for side in &world.player.borrow().bounding_box {
        draw_between(canvas, side.a.subtract(&offset), side.b.subtract(&offset));
    }

    // Draw look direction.
    canvas.set_draw_color(Color::RGB(255, 0, 0));
    let end = world.player.borrow().pos.add(&world.player.borrow().look_direction.scale(half_player_size as f64));
    draw_between(canvas, world.player.borrow().pos.subtract(&offset), end.subtract(&offset));
}

fn draw_wall_2d(canvas: &mut WindowCanvas, offset: Vector2, wall: &Wall, contains_the_player: bool) {
    let color = if contains_the_player {
        if wall.is_portal() {
            Color::RGB(0, 255, 255)
        } else {
            Color::RGB(0, 255, 0)
        }
    } else {
        if wall.is_portal() {
            Color::RGB(0, 155, 15)
        } else {
            Color::RGB(0, 0, 255)
        }
    };

    canvas.set_draw_color(color);
    draw_between(canvas, wall.line.a.subtract(&offset), wall.line.b.subtract(&offset));

    // Draw normal
    canvas.set_draw_color(Color::RGB(200, 0, 200));
    draw_between(canvas, wall.line.middle().subtract(&offset), wall.line.middle().add(&wall.normal.scale(5.0)).subtract(&offset));
}

fn draw_ray_segment_2d(canvas: &mut WindowCanvas, offset: Vector2, segment: &HitResult, hit_colour: Colour, miss_colour: Colour) {
    match segment.kind {
        HitKind::Wall { .. }
         | HitKind::Player { .. } => {
            canvas.set_draw_color(hit_colour.sdl());
            draw_between(canvas, segment.line.a.subtract(&offset), segment.line.b.subtract(&offset));
        }
        HitKind::None => {
            canvas.set_draw_color(miss_colour.sdl());
            draw_between(canvas, segment.line.a.subtract(&offset), segment.line.a.add(&segment.line.direction().normalize().scale(-100.0)).subtract(&offset));
        }
    }
}

pub(crate) fn render3d(world: &World, canvas: &mut WindowCanvas, _delta_time: f64){
    for x in 0..((SCREEN_WIDTH as f64 * RESOLUTION_FACTOR) as i32) {
        let x = (x as f64 / RESOLUTION_FACTOR) as i32;
        let look_direction = ray_direction_for_x(x, &world.player.borrow().look_direction);
        let segments = ray_trace(world.player.borrow().pos, look_direction, &world.player.borrow().region);

        let mut cumulative_dist = 0.0;
        for segment in &segments {
            draw_floor_segment(canvas, segment, x, cumulative_dist);
            cumulative_dist += segment.line.length();
        }

        draw_wall_3d(&world.player.borrow(), canvas, segments.last().unwrap(), look_direction, cumulative_dist, x);
    }
}

fn draw_floor_segment(canvas: &mut WindowCanvas, segment: &HitResult, screen_x: i32, cumulative_dist: f64){
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

            let t = i as f64 / steps_per_sample;
            let colour = current.lerp(&next, t);  // TODO: its a quadratic not a line.
            canvas.set_draw_color(colour.sdl());
            draw_between(canvas, Vector2::of(screen_x as f64, bottom), Vector2::of(screen_x as f64, top));

            last_top = top;
        }
    }
}

// the sample_count should be high enough that we have one past the end to lerp to
fn light_floor_segment(segment: &HitResult, sample_length: f64, sample_count: i32) -> Vec<Colour> {
    let ray_line = segment.line;
    let region = segment.region.upgrade().unwrap();
    let mut samples: Vec<Colour> = Vec::with_capacity((sample_count) as usize);
    for i in 0..sample_count {
        let pos = ray_line.a.add(&ray_line.direction().normalize().scale(i as f64 * -sample_length));
        samples.push(light_floor_point(&region, pos));
    }

    samples
}

fn light_floor_point(region: &Rc<RefCell<Region>>, hit_pos: Vector2) -> Colour {
    let mut colour = Colour::black();
    for light in &region.borrow().lights {
        colour = colour.add(region.borrow().floor_material.direct_floor_lighting(region, light, hit_pos));
    }
    for wall in &region.borrow().walls {
        for (light, fake_location) in &wall.borrow().lights {
            colour = colour.add(region.borrow().floor_material.portal_floor_lighting(region, *fake_location, wall.borrow().line, &light, hit_pos));
        }
    }

    colour
}

fn draw_wall_3d(player: &Player, canvas: &mut WindowCanvas, hit: &HitResult, ray_direction: Vector2, cumulative_dist: f64, screen_x: i32) {
    let hit_point = hit.line.b;
    let wall_normal = match &hit.kind {
        HitKind::None { .. } => { ray_direction }
        HitKind::Wall { hit_wall, .. } => {
            hit_wall.upgrade().unwrap().borrow().normal
        }
        HitKind::Player { box_side, .. } => {
            box_side.normal()
        }
    };

    let material = match &hit.kind {
        HitKind::None { .. } => { Material::new(0.0, 0.0, 0.0) }
        HitKind::Wall { hit_wall, .. } => {
            hit_wall.upgrade().unwrap().borrow().material
        }
        HitKind::Player { box_side, .. } => {
            player.material
        }
    };

    let colour= light_wall_column(&hit.region.upgrade().unwrap(), &hit_point, wall_normal, &material, player, ray_direction, screen_x);
    let (top, bottom) = project_to_screen(cumulative_dist);

    canvas.set_draw_color(colour.sdl());
    draw_between(canvas, Vector2::of(screen_x as f64, top), Vector2::of(screen_x as f64, bottom));
}


fn light_wall_column(region: &Rc<RefCell<Region>>, hit_point: &Vector2, wall_normal: Vector2, material: &Material, player: &Player, ray_direction: Vector2, x: i32) -> Colour {
    let middle = SCREEN_WIDTH as i32 / 2;

    let is_in_middle_half = (x - middle).abs() < (middle / 2);
    let has_flash_light = player.has_flash_light && is_in_middle_half;

    let mut colour = Colour::black();
    let to_eye = ray_direction.negate().normalize();
    for light in &region.borrow().lights {
        colour = colour.add(material.direct_wall_lighting(region, &light, hit_point, wall_normal, &to_eye));
    }

    for wall in &region.borrow().walls {
        for (light, fake_location) in &wall.borrow().lights {
            colour = colour.add(material.portal_wall_lighting(region, *fake_location, wall.borrow().line, &light, hit_point, wall_normal, &to_eye));
        }
    }

    if has_flash_light && x == middle {
        return colour.multiply(Colour::new(1.0, 0.5, 0.5));
    }

    colour
}

/// Converts a (distance to a wall) into a top and bottom y to draw that wall on the canvas.
/// https://nicolbolas.github.io/oldtut/Positioning/Tut04%20Perspective%20Projection.html
fn project_to_screen(z_distance: f64) -> (f64, f64) {
    let zoom_amount = 12000.0;
    let screen_wall_height = zoom_amount / z_distance;
    let screen_middle = SCREEN_HEIGHT / 2.0;
    let half_screen_wall_height = screen_wall_height / 2.0;
    let y_top = (screen_middle - half_screen_wall_height).max(0.0);
    let y_bottom = (screen_middle + half_screen_wall_height).min(SCREEN_HEIGHT - 1.0);
    (y_top, y_bottom)
}

fn x_to_angle(screen_x: i32) -> f64{
    let t = screen_x as f64 / SCREEN_WIDTH as f64;
    let delta_deg = (t - 0.5) * FOV_DEG as f64;
    let delta_rad = PI * delta_deg / 180.0;
    delta_rad
}

/// Assuming the forwards points at the middle of the screen, return it rotated to point at x instead.
pub(crate) fn ray_direction_for_x(screen_x: i32, forwards: &Vector2) -> Vector2 {
    forwards.rotate(x_to_angle(screen_x))
}

