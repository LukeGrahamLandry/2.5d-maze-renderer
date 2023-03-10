use std::f64::consts::PI;
use sdl2::keyboard::Scancode::V;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use crate::mth::{LineSegment2, Vector2};

use crate::world::{Wall, World};

const FOV_DEG: i32 = 45;
const VIEW_DIST: f64 = 1000.0;
const SCREEN_HEIGHT: f64 = 600.0;
const SCREEN_WIDTH: u32 = 800;
const COL_WIDTH: u32 = SCREEN_WIDTH / (FOV_DEG as u32);

pub(crate) fn render2d(world: &World, canvas: &mut WindowCanvas, _delta_time: f64){
    let half_player_size = 5;
    let x = world.player.pos.x as i32;
    let y = world.player.pos.y as i32;

    // Draw the player.
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.fill_rect(Rect::new(x - half_player_size, y - half_player_size, (half_player_size * 2) as u32, (half_player_size * 2) as u32)).unwrap();

    // Draw the walls.
    let mut i = 0;
    for region in world.regions.iter() {
        for wall in region.walls.iter() {
            if world.player.region_index == i {
                canvas.set_draw_color(Color::RGBA(200, 0, 200, 255));
                canvas.draw_line(wall.line.middle().sdl(), wall.line.middle().add(&wall.line.normal().scale(5.0)).sdl()).unwrap();
                if wall.has_next {
                    canvas.set_draw_color(Color::RGBA(0, 255, 255, 255));
                } else {
                    canvas.set_draw_color(Color::RGBA(0, 255, 0, 255));
                }
            } else {
                if wall.has_next {
                    canvas.set_draw_color(Color::RGBA(0, 155, 155, 255));
                } else {
                    canvas.set_draw_color(Color::RGBA(0, 0, 255, 255));
                }

            }

            canvas.draw_line(wall.line.a.sdl(), wall.line.b.sdl()).unwrap();
        }

        // Draw light
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.draw_point(region.light_pos.sdl()).unwrap();

        i += 1;
    }

    // Draw view rays.
    for delta_deg in -(FOV_DEG / 2)..(FOV_DEG / 2) {
        let delta_rad = PI * (delta_deg as f64) / 180.0;
        let look_direction = world.player.look_direction.rotate(delta_rad);
        let mut segments = vec![];
        let hit = ray_trace(&world, world.player.pos, look_direction, world.player.region_index, &mut segments);

        match hit {
            None => {
                canvas.set_draw_color(Color::RGBA(150, 150, 150, 255));
            }
            Some(hit) => {
                canvas.set_draw_color(Color::RGBA(150, 150, 0, 255));
            }
        }

        for segment in &segments {
            canvas.draw_line(segment.a.sdl(), segment.b.sdl()).unwrap();
        }
    }

    // Draw the player's collision ray.
    canvas.set_draw_color(Color::RGBA(255, 0, 0, 255));
    let player_view_end = world.player.pos.add(&world.player.look_direction.scale((half_player_size) as f64));
    let player_view_back_end = world.player.pos.add(&world.player.look_direction.negate().scale((half_player_size) as f64));
    canvas.draw_line(player_view_back_end.sdl(), player_view_end.sdl()).unwrap();
}

pub(crate) fn render3d(world: &World, canvas: &mut WindowCanvas, _delta_time: f64, mouse_pos: &Vector2){
    for x in 0..(SCREEN_WIDTH as i32) {
        let delta_deg = ((x as f64 / SCREEN_WIDTH as f64) - 0.5) * FOV_DEG as f64;
        let delta_rad = PI * delta_deg / 180.0;
        let look_direction = world.player.look_direction.rotate(delta_rad);
        let mut segments = vec![];
        let hit = ray_trace(&world, world.player.pos, look_direction, world.player.region_index, &mut segments);

        match hit {
            None => {
                canvas.set_draw_color(Color::RGB(100, 100, 100));
                canvas.draw_line(Vector2::of(x as f64, 0.0).sdl(), Vector2::of(x as f64, SCREEN_HEIGHT).sdl()).unwrap();
            }
            Some(hit) => {
                let region = &world.regions[hit.region_index];
                let hit_wall = &region.walls[hit.wall_index];

                let to_light = region.light_pos.subtract(&hit.point).normalize();
                let world_light_factor = hit_wall.line.normal().dot(&to_light).abs() * region.light_intensity;

                let flash_light_factor = if world.player.has_flash_light && (x - SCREEN_WIDTH as i32 / 2).abs() < (SCREEN_WIDTH as i32 / 4){
                    let imaginary_light = world.player.pos.clone();
                    let to_light = imaginary_light.subtract(&hit.point);
                    hit_wall.line.normal().dot(&to_light).abs()
                } else { 0.0 };

                let total_color_factor = world_light_factor + (flash_light_factor / 200.0);
                let full_color = (10.0 + (200.0 * total_color_factor)).min(255.0) as u8;

                canvas.set_draw_color(Color::RGB((10.0 * flash_light_factor) as u8, full_color, 0));

                let h = SCREEN_HEIGHT / hit.distance * 20.0;
                let top = ((SCREEN_HEIGHT / 2.0) - (h / 2.0)).max(0.0);
                let bottom = ((SCREEN_HEIGHT / 2.0) + (h / 2.0)).min(SCREEN_HEIGHT - 1.0);

                canvas.draw_line(Vector2::of(x as f64, top).sdl(), Vector2::of(x as f64, bottom).sdl()).unwrap();

                canvas.set_draw_color(region.floor_color);
                canvas.draw_line(Vector2::of(x as f64, bottom).sdl(), Vector2::of(x as f64, SCREEN_HEIGHT).sdl()).unwrap();
            }
        }
    }
}

fn ray_trace(world: &World, origin: Vector2, direction: Vector2, region_index: usize, segments: &mut Vec<LineSegment2>) -> Option<HitResult> {
    let region = &world.regions[region_index];

    let mut dist = f64::INFINITY;
    let mut first_hit = Vector2::NAN;
    let ray = LineSegment2::from(origin, direction.scale(VIEW_DIST));
    let mut i = 0;
    let mut first_hit_index = 0;
    for wall in &region.walls {
        let hit = wall.line.intersection(&ray);
        let to_hit = origin.subtract(&hit);
        if !hit.is_nan() && to_hit.length() < dist {
            dist = to_hit.length();
            first_hit = hit;
            first_hit_index = i;
        }
        i += 1;
    }

    if dist.is_infinite() {
        return None;
    }

    let hit_wall = &region.walls[first_hit_index];
    segments.push(LineSegment2::of(origin, first_hit));

    if !hit_wall.has_next {
        return Some(HitResult {
            region_index,
            wall_index: first_hit_index,
            point: first_hit,
            distance: dist,
        });
    }

    if segments.len() >= 5 {
        return None;
    }

    let new_region_index = hit_wall.next_region.unwrap();
    let new_wall_index = hit_wall.next_wall.unwrap();
    let new_wall = &world.regions[new_region_index].walls[new_wall_index];
    let new_origin = Wall::translate(&first_hit, hit_wall, new_wall);
    ray_trace(world, new_origin.add(&direction), direction, new_region_index, segments)
}

struct HitResult {
    region_index: usize,
    wall_index: usize,
    point: Vector2,
    distance: f64
}
