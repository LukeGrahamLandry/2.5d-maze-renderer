use std::f64::consts::PI;
use std::sync::{mpsc};
use std::{thread};
use sdl2::render::WindowCanvas;
use crate::camera::*;
use crate::material::{Colour};
use crate::mth::{LineSegment2, Vector2};
use crate::ray::{HitKind, RaySegment, ray_trace, single_ray_trace, trace_clear_portal_light};
use crate::world_data::{ColumnLight, Region, Wall, World};


pub(crate) fn render(world: &World, window: &mut WindowCanvas, _delta_time: f64){
    let player_offset = world.player.peek().pos.subtract(&Vector2::of((SCREEN_WIDTH / 2) as f64, SCREEN_HEIGHT / 2.0));
    let (sender, receiver) = mpsc::channel();

    thread::scope(|s| {
        {
            let sender = sender;
            s.spawn(move || {
                let sender = sender.clone();
                let mut handler = |line| {
                    sender.send(line).expect("Failed to send line.");
                };

                let mut canvas = RenderBuffer::new(&mut handler);
                canvas.offset = player_offset.negate();
                inner_render2d(world, &mut canvas, _delta_time);
            });
        }

        for line in receiver {
            window.set_draw_color(line.colour.to_u8());
            window.draw_line(line.a.sdl(), line.b.sdl()).expect("SDL draw failed.");
        }
    });
}

fn inner_render2d(world: &World, canvas: &mut RenderBuffer, _delta_time: f64){
    let half_player_size = 5;

    // Draw the regions.
    for region in world.regions.iter() {
        // Draw lights
        for light in region.peek().lights.iter() {
            let light = light.peek();
            let hit_colour = light.intensity.scale(0.3);
            let miss_colour = light.intensity.scale(0.1);
            for r in 0..LIGHT_RAY_COUNT_2D {
                // Draw rays
                let direction = Vector2::from_angle(r as f64 * PI / (LIGHT_RAY_COUNT_2D as f64 / 2.0), 1.0);
                let ray_start = light.pos.add(&direction.scale(3.0));
                let segment = single_ray_trace(ray_start, direction, region.peek());

                draw_ray_segment_2d(canvas, &segment, hit_colour, miss_colour);
            }
        }

        // Draw walls
        for wall in region.peek().iter_walls() {
            let contains_player = world.player.peek().region.is(&region);
            draw_wall_2d(canvas, &wall, contains_player);

            // Draw saved lights
            for light in wall.lights.iter() {
                draw_portal_light_2d(canvas, &wall, light.parent.peek(), &light.location, region.peek());
            }
        }
    }

    // Draw view rays.
    for x in 0..(SCREEN_WIDTH as i32) {
        if x % 15 != 0 {
            continue;
        }

        let look_direction = ray_direction_for_x(x, &world.player.peek().look_direction);
        let segments = ray_trace(world.player.peek().pos, look_direction, &world.player.peek().region.peek());
        let hit_colour = Colour::rgb(150, 150, 0);
        let miss_colour = Colour::rgb(150, 150, 150);
        for segment in &segments {
            draw_ray_segment_2d(canvas, segment, hit_colour, miss_colour);
        }
    }

    // Draw the player.
    canvas.set_draw_color(Colour::rgb(255, 255, 255));
    for side in &world.player.peek().bounding_box {
        canvas.draw_between(side.a, side.b);
    }

    // Draw look direction.
    canvas.set_draw_color(Colour::rgb(255, 0, 0));
    let end = world.player.peek().pos.add(&world.player.peek().look_direction.scale(half_player_size as f64));
    canvas.draw_between(world.player.peek().pos, end);
}

fn draw_portal_light_2d(canvas: &mut RenderBuffer, wall: &Wall, light: &ColumnLight, fake_location: &LineSegment2, region: &MapRegion) {
    let light_fake_origin = fake_location.get_b();
    let _light_hits_portal_at = fake_location.get_a();

    canvas.set_draw_color(light.intensity.scale(0.2));
    for r in 0..LIGHT_RAY_COUNT_2D {
        let direction = Vector2::from_angle(r as f64 * PI / (LIGHT_RAY_COUNT_2D as f64 / 2.0), 1.0);

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
                canvas.draw_between(line.a, line.b);
            }
        }
    }

    canvas.set_draw_color(light.intensity.scale(1.0));
    canvas.draw_between(fake_location.a, fake_location.b);
}



fn draw_wall_2d(canvas: &mut RenderBuffer, wall: &Wall, contains_the_player: bool) {
    let color = if contains_the_player {
        if wall.is_portal() {
            Colour::rgb(0, 255, 255)
        } else {
            Colour::rgb(0, 255, 0)
        }
    } else {
        if wall.is_portal() {
            Colour::rgb(0, 155, 15)
        } else {
            Colour::rgb(0, 0, 255)
        }
    };

    canvas.set_draw_color(color);
    canvas.draw_between(wall.line.a, wall.line.b);

    // Draw normal
    canvas.set_draw_color(Colour::rgb(200, 0, 200));
    canvas.draw_between(wall.line.middle(), wall.line.middle().add(&wall.normal.scale(5.0)));
}

fn draw_ray_segment_2d(canvas: &mut RenderBuffer, segment: &RaySegment, hit_colour: Colour, miss_colour: Colour) {
    match segment.kind {
        HitKind::HitWall { .. }
        | HitKind::HitPlayer { .. } => {
            canvas.set_draw_color(hit_colour);
            canvas.draw_between(segment.line.a, segment.line.b);
        }
        HitKind::HitNone => {
            canvas.set_draw_color(miss_colour);
            canvas.draw_between(segment.line.a, segment.line.a.add(&segment.line.direction().normalize().scale(-100.0)));
        }
    }
}
