use std::f64::consts::PI;
use sdl2::keyboard::Scancode::V;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use crate::mth::{LineSegment2, Vector2};

use crate::world::World;

const FOV_DEG: i32 = 90;
const VIEW_DIST: f64 = 200.0;
const SCREEN_HEIGHT: u32 = 600;
const SCREEN_WIDTH: u32 = 800;
const COL_WIDTH: u32 = SCREEN_WIDTH / (FOV_DEG as u32);

pub(crate) fn render2d(world: &World, canvas: &mut WindowCanvas, _delta_time: f64){
    let half_player_size = 5;
    let x = world.player.pos.x as i32;
    let y = world.player.pos.y as i32;

    // Draw the player.
    canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
    canvas.fill_rect(Rect::new(x - half_player_size, y - half_player_size, (half_player_size * 2) as u32, (half_player_size * 2) as u32)).expect("Draw failed");

    // Draw the walls.
    let mut i = 0;
    for region in world.regions.iter() {
        for wall in region.walls.iter() {
            if world.player.region_index == i {
                if wall.has_next {
                    canvas.set_draw_color(Color::RGBA(0, 255, 255, 255));
                } else {
                    canvas.set_draw_color(Color::RGBA(0, 255, 0, 255));
                }

            } else {
                canvas.set_draw_color(Color::RGBA(0, 0, 255, 255));
            }


            canvas.draw_line(wall.line.a.sdl(), wall.line.b.sdl()).expect("Draw failed");
        }

        i += 1;
    }

    // Draw view rays.
    let region = &world.regions[world.player.region_index];
    for delta_deg in -(FOV_DEG / 2)..(FOV_DEG / 2) {
        let delta_rad = PI * (delta_deg as f64) / 180.0;
        let mut dist = f64::INFINITY;
        let mut first_hit = Vector2::NAN;
        let view_vec = world.player.direction.rotate(delta_rad).scale(VIEW_DIST);
        let ray = LineSegment2::from(world.player.pos.clone(), view_vec);
        for wall in &region.walls {
            let hit = wall.line.intersection(&ray);
            let to_hit = world.player.pos.subtract(&hit);
            if !hit.is_nan() && to_hit.length() < dist {
                dist = to_hit.length();
                first_hit = hit;
            }
        }

        if dist.is_finite() {
            canvas.set_draw_color(Color::RGBA(150, 150, 0, 255));
        } else {
            canvas.set_draw_color(Color::RGBA(150, 150, 150, 255));
            first_hit = world.player.pos.add(&view_vec);
        }

        canvas.draw_line(world.player.pos.sdl(), first_hit.sdl()).expect("Draw failed");
    }

    // Draw the player's collision ray.
    canvas.set_draw_color(Color::RGBA(255, 0, 0, 255));
    let player_view_end = world.player.pos.add(&world.player.direction.scale((half_player_size * 2) as f64));
    canvas.draw_line(world.player.pos.sdl(), player_view_end.sdl()).expect("Draw failed");
}



pub(crate) fn render3d(world: &World, canvas: &mut WindowCanvas, _delta_time: f64){
    let region = &world.regions[world.player.region_index];
    for x in 0..(SCREEN_WIDTH as i32) {
        let delta_deg = ((x as f64 / SCREEN_WIDTH as f64) - 0.5) * FOV_DEG as f64;
        let delta_rad = PI * delta_deg / 180.0;
        let mut dist = f64::INFINITY;
        let mut first_hit = Vector2::NAN;
        let view_vec = world.player.direction.rotate(delta_rad).scale(VIEW_DIST);
        let ray = LineSegment2::from(world.player.pos.clone(), view_vec);
        let mut i = 0;
        let mut first_hit_index = 0;
        for wall in &region.walls {
            let hit = wall.line.intersection(&ray);
            let to_hit = world.player.pos.subtract(&hit);
            if !hit.is_nan() && to_hit.length() < dist {
                dist = to_hit.length();
                first_hit = hit;
                first_hit_index = i;
            }
            i += 1;
        }

        if dist.is_finite() {
            if world.regions[world.player.region_index].walls[first_hit_index].has_next {
                canvas.set_draw_color(Color::RGBA(0, 255, 255, 255));
            } else {
                canvas.set_draw_color(Color::RGBA(0, 255, 0, 255));
            }
        } else {
            canvas.set_draw_color(Color::RGBA(150, 150, 150, 255));
            first_hit = world.player.pos.add(&view_vec);
            dist = VIEW_DIST;
        }

        let height = dist * ((PI / 4.0) as f64).tan();
        println!("{} {}", dist, height);
        canvas.draw_line(Vector2::of(x as f64, SCREEN_HEIGHT as f64).sdl(), Vector2::of(x as f64, (SCREEN_HEIGHT - height as u32) as f64).sdl()).expect("Draw failed");
    }
}
