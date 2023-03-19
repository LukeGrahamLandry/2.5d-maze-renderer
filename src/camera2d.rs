use std::f64::consts::PI;
use std::sync::{mpsc};
use std::{thread};
use sdl2::render::WindowCanvas;
use crate::camera::*;
use crate::light_cache::{LightingRegion, PortalLight};
use crate::lighting::LightSource;
use crate::material::{Colour};
use crate::mth::{LineSegment2, Vector2};
use crate::new_world::World;
use crate::ray::{RaySegment, SolidWall};


pub(crate) fn render<'map: 'walls, 'walls>(world: &'map World<'map> , window: &mut WindowCanvas, _delta_time: f64){
    let player_offset = world.player().entity.pos.subtract(&Vector2::of((SCREEN_WIDTH / 2) as f64, SCREEN_HEIGHT / 2.0));
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

fn inner_render2d<'map: 'walls, 'walls>(world: &'map World<'map> , canvas: &mut RenderBuffer, _delta_time: f64){
    let half_player_size = 5;

    // Draw the regions.
    for region in &world.get_light_cache().lights {
        // Draw direct lights
        for light in region.region.lights() {
            let hit_colour = light.intensity().scale(0.3);
            let miss_colour = light.intensity().scale(0.1);
            for r in 0..LIGHT_RAY_COUNT_2D {
                // Draw rays
                let direction = Vector2::from_angle(r as f64 * PI / (LIGHT_RAY_COUNT_2D as f64 / 2.0), 1.0);
                let ray_start = light.apparent_pos().add(&direction.scale(3.0));
                let segment = region.single_ray_trace(ray_start, direction);

                draw_ray_segment_2d(canvas, &segment, hit_colour, miss_colour);
            }
        }

        // Draw portal lights
        for light in &region.portal_lights {
            draw_portal_light_2d(region, canvas, light);
        }

        // Draw walls
        for wall in region.region.walls() {
            let contains_player = world.player().entity.region == wall.region;
            draw_wall_2d(canvas, wall, contains_player);
        }
    }

    // Draw view rays.
    for x in 0..(SCREEN_WIDTH as i32) {
        if x % 15 != 0 {
            continue;
        }

        let look_direction = ray_direction_for_x(x, &world.player().look_direction);
        let region = world.get_light_cache().get_lighting_region(world.player().entity.region);
        let segments = world.get_light_cache().ray_trace(region, world.player().entity.pos, look_direction);
        let hit_colour = Colour::rgb(150, 150, 0);
        let miss_colour = Colour::rgb(150, 150, 150);
        for segment in &segments {
            draw_ray_segment_2d(canvas, segment, hit_colour, miss_colour);
        }
    }

    // Draw the player.
    canvas.set_draw_color(Colour::rgb(255, 255, 255));
    for side in &world.player().entity.get_bounding_box() {
        canvas.draw_between(side.line().a, side.line().b);
    }

    // Draw look direction.
    canvas.set_draw_color(Colour::rgb(255, 0, 0));
    let end = world.player().entity.pos.add(&world.player().look_direction.scale(half_player_size as f64));
    canvas.draw_between(world.player().entity.pos, end);
}

fn draw_portal_light_2d<'map: 'walls, 'walls>(region: &'walls LightingRegion<'map, 'walls>, canvas: &mut RenderBuffer, light: &'walls PortalLight<'map, 'walls>) {
    let wall = light.portal_out;
    canvas.set_draw_color(light.intensity().scale(0.2));
    for r in 0..LIGHT_RAY_COUNT_2D {
        let direction = Vector2::from_angle(r as f64 * PI / (LIGHT_RAY_COUNT_2D as f64 / 2.0), 1.0);

        let light_toward_portal = LineSegment2::from(*light.apparent_pos(), direction.scale(100.0));
        let point_on_portal = wall.line().algebraic_intersection(&light_toward_portal);
        let actually_crosses_portal = wall.line().contains(&point_on_portal);
        if !actually_crosses_portal {
            continue;
        }

        let direction = point_on_portal.subtract(light.apparent_pos());
        let segment = region.single_ray_trace(point_on_portal.add(&direction.tiny()), direction);
        let line = region.trace_clear_portal_light(light, segment.line.get_b());
        match line {
            None => {}
            Some(line) => {
                canvas.draw_between(line.a, line.b);
            }
        }
    }

    canvas.set_draw_color(light.intensity().scale(1.0));
    canvas.draw_between(wall.line().middle(), *light.apparent_pos());
}



fn draw_wall_2d<'map, 'walls>(canvas: &mut RenderBuffer, wall: &'walls dyn SolidWall<'map, 'walls>, contains_the_player: bool) {
    let color = if contains_the_player {
        if wall.portal().is_some() {
            Colour::rgb(0, 255, 255)
        } else {
            Colour::rgb(0, 255, 0)
        }
    } else {
        if wall.portal().is_some() {
            Colour::rgb(0, 155, 15)
        } else {
            Colour::rgb(0, 0, 255)
        }
    };

    canvas.set_draw_color(color);
    canvas.draw_between(wall.line().a, wall.line().b);

    // Draw normal
    canvas.set_draw_color(Colour::rgb(200, 0, 200));
    canvas.draw_between(wall.line().middle(), wall.line().middle().add(&wall.normal().scale(5.0)));
}

fn draw_ray_segment_2d(canvas: &mut RenderBuffer, segment: &RaySegment, hit_colour: Colour, miss_colour: Colour) {
    match segment.hit_wall {
        None => {
            canvas.set_draw_color(miss_colour);
            canvas.draw_between(segment.line.a, segment.line.a.add(&segment.line.direction().normalize().scale(-100.0)));
        }
        Some(_) => {
            canvas.set_draw_color(hit_colour);
            canvas.draw_between(segment.line.a, segment.line.b);
        }
    }
}
