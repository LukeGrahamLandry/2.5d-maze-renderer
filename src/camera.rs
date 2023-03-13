use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::{Rc, Weak};
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::player::Player;
use crate::ray::{HitKind, HitResult, ray_trace};

use crate::world::{Region, Wall, World};

const FOV_DEG: i32 = 45;
const SCREEN_HEIGHT: f64 = 600.0;
pub const SCREEN_WIDTH: u32 = 800;

pub(crate) fn render2d(world: &World, canvas: &mut WindowCanvas, _delta_time: f64){
    let half_player_size = 5;
    let x = world.player.borrow().pos.x as i32;
    let y = world.player.borrow().pos.y as i32;

    // Draw the regions.
    let mut i = 0;
    for region in world.regions.iter() {
        for wall in region.borrow().walls.iter() {
            let contains_player = Rc::ptr_eq(&world.player.borrow().region, region);
            draw_wall_2d(canvas, &wall.borrow(), contains_player);
        }

        // Draw light
        for light in &region.borrow().lights {
            canvas.set_draw_color(light.intensity.sdl());
            canvas.draw_point(light.pos.sdl()).unwrap();
        }

        i += 1;
    }

    // Draw view rays.
    for x in 0..(SCREEN_WIDTH as i32) {
        if x % 15 != 0 {
            continue;
        }

        let look_direction = ray_direction_for_x(x, &world.player.borrow().look_direction);
        let segments = ray_trace(world.player.borrow().pos, look_direction, &world.player.borrow().region);

        for segment in &segments {
            draw_ray_segment_2d(canvas, segment);
        }
    }

    // Draw the player.
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for side in &world.player.borrow().bounding_box {
        canvas.draw_line(side.a.sdl(), side.b.sdl()).unwrap();
    }

    // Draw look direction.
    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.draw_line(world.player.borrow().pos.sdl(), world.player.borrow().pos.add(&world.player.borrow().look_direction.scale(half_player_size as f64)).sdl()).unwrap();
}

fn draw_wall_2d(canvas: &mut WindowCanvas, wall: &Wall, contains_the_player: bool) {
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
    canvas.draw_line(wall.line.a.sdl(), wall.line.b.sdl()).unwrap();

    // Draw normal
    canvas.set_draw_color(Color::RGB(200, 0, 200));
    canvas.draw_line(wall.line.middle().sdl(), wall.line.middle().add(&wall.normal.scale(5.0)).sdl()).unwrap();
}

fn draw_ray_segment_2d(canvas: &mut WindowCanvas, segment: &HitResult) {
    match segment.kind {
        HitKind::Wall { .. }
         | HitKind::Player { .. } => {
            canvas.set_draw_color(Color::RGB(150, 150, 0));
            canvas.draw_line(segment.line.a.sdl(), segment.line.b.sdl()).unwrap();
        }
        HitKind::None => {
            canvas.set_draw_color(Color::RGB(150, 150, 150));
            canvas.draw_line(segment.line.a.sdl(), segment.line.a.add(&segment.line.direction().normalize().scale(-100.0)).sdl()).unwrap();
        }
    }
}

pub(crate) fn render3d(world: &World, canvas: &mut WindowCanvas, _delta_time: f64){
    for x in 0..(SCREEN_WIDTH as i32) {
        let look_direction = ray_direction_for_x(x, &world.player.borrow().look_direction);
        let segments = ray_trace(world.player.borrow().pos, look_direction, &world.player.borrow().region);

        let mut cumulative_dist = 0.0;
        for segment in &segments {
            draw_floor_segment(canvas, &segment.region.upgrade().unwrap().borrow(), segment.line, x, cumulative_dist);
            cumulative_dist += segment.line.length();
        }

        draw_wall_3d(&world.player.borrow(), canvas, segments.last().unwrap(), look_direction, cumulative_dist, x);
    }
}

fn draw_floor_segment(canvas: &mut WindowCanvas, region: &Region, ray_line: LineSegment2, screen_x: i32, mut cumulative_dist: f64){
    // The top of the last floor segment is the bottom of this one.
    let (pixels_drawn, _) = project_to_screen(cumulative_dist);
    let mut bottom = SCREEN_HEIGHT - pixels_drawn;

    // The top of the floor segment is the bottom of where we'd draw if it was a wall.
    let length = ray_line.length();
    let sample_length = 10.0;
    let sample_count = (length / sample_length) as i32;
    let mut samples: Vec<Colour> = Vec::with_capacity((sample_count + 1) as usize);
    for i in 0..(sample_count + 1) {  // make sure we have one past the end to lerp to
        let t = i as f64 / sample_count as f64;
        let pos = ray_line.a.add(&ray_line.direction().scale(-t));

        samples.push(floor_point_lighting(region, pos));
    }

    for s in 0..(sample_count + 1) {
        let current = samples[s as usize];
        let (_, top) = project_to_screen(cumulative_dist + sample_length);
        canvas.set_draw_color(current.sdl());
        canvas.draw_line(Vector2::of(screen_x as f64, bottom).sdl(), Vector2::of(screen_x as f64, top).sdl()).unwrap();
        cumulative_dist += sample_length;
        bottom -= (bottom - top);
    }
}

fn floor_point_lighting(region: &Region, hit_pos: Vector2) -> Colour {
    let mut colour = Colour::black();
    for light in &region.lights {
        colour = colour.add(region.floor_material.floor_lighting(light, hit_pos));
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

    let colour= wall_column_lighting(&hit.region.upgrade().unwrap().borrow(), &hit_point, &wall_normal, &material, player, ray_direction, screen_x);
    let (top, bottom) = project_to_screen(cumulative_dist);

    canvas.set_draw_color(colour.sdl());
    canvas.draw_line(Vector2::of(screen_x as f64, top).sdl(), Vector2::of(screen_x as f64, bottom).sdl()).unwrap();
}


fn wall_column_lighting(region: &Region, hit_point: &Vector2, wall_normal: &Vector2, material: &Material, player: &Player, ray_direction: Vector2, x: i32) -> Colour {
    let middle = SCREEN_WIDTH as i32 / 2;

    let is_in_middle_half = (x - middle).abs() < (middle / 2);
    let has_flash_light = player.has_flash_light && is_in_middle_half;

    let mut colour = Colour::black();
    let to_eye = ray_direction.negate().normalize();
    for light in &region.lights {
        colour = colour.add(material.lighting(&light, hit_point, wall_normal, &to_eye));
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

/// Assuming the forwards points at the middle of the screen, return it rotated to point at x instead.
pub(crate) fn ray_direction_for_x(screen_x: i32, forwards: &Vector2) -> Vector2 {
    let t = screen_x as f64 / SCREEN_WIDTH as f64;
    let delta_deg = (t - 0.5) * FOV_DEG as f64;
    let delta_rad = PI * delta_deg / 180.0;
    forwards.rotate(delta_rad)
}

