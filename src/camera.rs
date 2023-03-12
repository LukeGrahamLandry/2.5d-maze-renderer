use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::{Rc, Weak};
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;
use crate::mth::{LineSegment2, Vector2};
use crate::player::Player;

use crate::world::{Region, Wall, World};

const FOV_DEG: i32 = 45;
const VIEW_DIST: f64 = 1000.0;
const SCREEN_HEIGHT: f64 = 600.0;
pub const SCREEN_WIDTH: u32 = 800;
const PORTAL_LIMIT: u16 = 5;

pub(crate) fn render2d(world: &World, canvas: &mut WindowCanvas, _delta_time: f64){
    let half_player_size = 5;
    let x = world.player.pos.x as i32;
    let y = world.player.pos.y as i32;

    // Draw the regions.
    let mut i = 0;
    for region in world.regions.iter() {
        for wall in region.borrow().walls.iter() {
            let contains_player = Rc::ptr_eq(&world.player.region, region);
            draw_wall_2d(canvas, &wall.borrow(), contains_player);
        }

        // Draw light
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.draw_point(region.borrow().light_pos.sdl()).unwrap();

        i += 1;
    }

    // Draw view rays.
    for x in 0..(SCREEN_WIDTH as i32) {
        let look_direction = ray_direction_for_x(x, &world.player.look_direction);
        let segments = ray_trace(world.player.pos, look_direction, &world.player.region);

        for segment in &segments {
            draw_ray_segment_2d(canvas, segment);
        }
    }

    // Draw the player.
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for i in 0..(half_player_size * 2) {
        let x = (x - half_player_size + i) as f64;
        canvas.draw_line(Vector2::of(x, (y - half_player_size) as f64).sdl(), Vector2::of(x, (y + half_player_size) as f64).sdl()).unwrap();
    }

    // Draw look direction.
    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.draw_line(world.player.pos.sdl(), world.player.pos.add(&world.player.look_direction.scale(half_player_size as f64)).sdl()).unwrap();
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
    match segment.hit_wall {
        Some(_) => {
            canvas.set_draw_color(Color::RGB(150, 150, 0));
            canvas.draw_line(segment.line.a.sdl(), segment.line.b.sdl()).unwrap();
        }
        None => {
            canvas.set_draw_color(Color::RGB(150, 150, 150));
            canvas.draw_line(segment.line.a.sdl(), segment.line.a.add(&segment.line.direction().normalize().scale(-100.0)).sdl()).unwrap();
        }
    }
}

pub(crate) fn render3d(world: &World, canvas: &mut WindowCanvas, _delta_time: f64){
    for x in 0..(SCREEN_WIDTH as i32) {
        let look_direction = ray_direction_for_x(x, &world.player.look_direction);
        let segments = ray_trace(world.player.pos, look_direction, &world.player.region);

        let mut cumulative_dist = 0.0;
        for segment in &segments {
            draw_floor_segment(canvas, &segment.region.upgrade().unwrap().borrow(), segment.line.length(), x, cumulative_dist);
            cumulative_dist += segment.line.length();
        }

        draw_wall_3d(&world.player, canvas, segments.last().unwrap(), look_direction, cumulative_dist, x);
    }
}

fn draw_floor_segment(canvas: &mut WindowCanvas, region: &Region, length: f64, screen_x: i32, cumulative_dist: f64){
    // The top of the last floor segment is the bottom of this one.
    let (pixels_drawn, _) = project_to_screen(cumulative_dist);
    let bottom = SCREEN_HEIGHT - pixels_drawn;

    // The top of the floor segment is the bottom of where we'd draw if it was a wall.
    let (_, top) = project_to_screen(cumulative_dist + length);

    canvas.set_draw_color(region.floor_color);
    canvas.draw_line(Vector2::of(screen_x as f64, bottom).sdl(), Vector2::of(screen_x as f64, top).sdl()).unwrap();
}

fn draw_wall_3d(player: &Player, canvas: &mut WindowCanvas, hit: &HitResult, player_look_direction: Vector2, cumulative_dist: f64, screen_x: i32) {
    let hit_point = hit.line.b;
    let wall_normal = match &hit.hit_wall {
        None => { player_look_direction }
        Some(hit_wall) => {
            hit_wall.upgrade().unwrap().borrow().normal
        }
    };

    let (red, green, blue) = wall_column_lighting(&hit.region.upgrade().unwrap().borrow(), &hit_point, &wall_normal, &player, screen_x);
    let (top, bottom) = project_to_screen(cumulative_dist);

    canvas.set_draw_color(Color::RGB(red, green, blue));
    canvas.draw_line(Vector2::of(screen_x as f64, top).sdl(), Vector2::of(screen_x as f64, bottom).sdl()).unwrap();
}

/// Returns the colour of a certain point on the wall.
fn wall_column_lighting(region: &Region, hit_point: &Vector2, wall_normal: &Vector2, player: &Player, x: i32) -> (u8, u8, u8) {
    let to_light = region.light_pos.subtract(&hit_point).normalize();
    let world_light_factor = wall_normal.dot(&to_light).abs() * region.light_intensity;

    let is_in_middle_half = (x - SCREEN_WIDTH as i32 / 2).abs() < (SCREEN_WIDTH as i32 / 4);
    let flash_light_factor = if player.has_flash_light && is_in_middle_half{
        let imaginary_light = player.pos.clone();
        let to_light = imaginary_light.subtract(&hit_point);
        wall_normal.dot(&to_light).abs()
    } else { 0.0 };

    let total_color_factor = world_light_factor + (flash_light_factor / 200.0);
    let full_color = (10.0 + (200.0 * total_color_factor)).min(255.0) as u8;
    ((10.0 * flash_light_factor) as u8, full_color, 0)
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

/// Sends a ray through the world, following portals, and returns a separate line segment for each region it passes through.
pub(crate) fn ray_trace(mut origin: Vector2, mut direction: Vector2, region: &Rc<RefCell<Region>>) -> Vec<HitResult> {
    let mut segments = vec![];

    let mut segment = single_ray_trace(origin, direction, region);
    for _ in 0..PORTAL_LIMIT {
        match &segment.hit_wall {
            None => { break; }
            Some(hit_wall) => {
                let wall = hit_wall.upgrade().unwrap();
                let wall = wall.borrow();

                match &wall.next_wall {
                    None => { break; }
                    Some(new_wall) => {
                        let t = wall.line.t_of(&segment.line.b).abs();
                        let hit_back = wall.normal.dot(&direction) > 0.0;
                        let hit_edge = t < 0.01 || t > 0.99;
                        if hit_back || hit_edge {
                            break;
                        }

                        // Go through the portal
                        let new_wall = new_wall.upgrade().unwrap();
                        let new_wall = new_wall.borrow();
                        origin = Wall::translate(segment.line.b, &wall, &new_wall);
                        direction = Wall::rotate(direction, &wall, &new_wall);

                        segments.push(segment);
                        segment = single_ray_trace(origin.add(&direction), direction, &new_wall.region.upgrade().unwrap());
                    }
                }
            }
        }
    }

    segments.push(segment);
    segments
}

/// Sends a ray through a single region until it hits a wall.
fn single_ray_trace(origin: Vector2, direction: Vector2, region: &Rc<RefCell<Region>>) -> HitResult {
    let ray = LineSegment2::from(origin, direction.scale(VIEW_DIST));

    let mut shortest_hit_distance = f64::INFINITY;
    let mut closest_hit_point = Vector2::NAN;
    let mut hit_wall = None;


    let m_region = region.borrow();
    for wall in &m_region.walls {
        let hit = wall.borrow().line.intersection(&ray);
        let to_hit = origin.subtract(&hit);

        if !hit.is_nan() && to_hit.length() < shortest_hit_distance {
            hit_wall = Some(wall);
            shortest_hit_distance = to_hit.length();
            closest_hit_point = hit;
        }
    }

    match hit_wall {
        None => {
            HitResult {
                region: Rc::downgrade(region),
                hit_wall: None,
                line: LineSegment2::of(origin, origin.add(&direction.scale(VIEW_DIST))),
            }
        }
        Some(hit_wall) => {
            HitResult {
                region: Rc::downgrade(region),
                hit_wall: Some(Rc::downgrade(&hit_wall)),
                line: LineSegment2::of(origin, closest_hit_point)
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct HitResult {
    pub(crate) region: Weak<RefCell<Region>>,
    pub(crate) hit_wall: Option<Weak<RefCell<Wall>>>,
    pub(crate) line: LineSegment2
}
