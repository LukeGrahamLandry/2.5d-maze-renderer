use crate::camera::*;
use crate::material::Colour;
use crate::mth::{LineSegment2, Vector2};
use crate::ray::RaySegment;
use crate::world::{LightKind, LightSource, Region, Wall, World};
use std::f64::consts::PI;

pub(crate) fn render<R: RenderStrategy>(world: &World, canvas: &mut R) {
    // let player_offset = world
    //     .player()
    //     .entity
    //     .pos
    //     .subtract(&Vector2::of((SCREEN_WIDTH / 2) as f64, SCREEN_HEIGHT / 2.0));
    // canvas.offset = player_offset.negate();
    inner_render2d(world, canvas);
}

fn inner_render2d<R: RenderStrategy>(world: &World, canvas: &mut R) {
    let half_player_size = 5;

    // Draw the regions.
    for region in world.regions() {
        // Draw lights
        for light in region.lights() {
            match &light.kind {
                LightKind::DIRECT() => {
                    let colour = light.intensity.scale(0.3);
                    for r in 0..LIGHT_RAY_COUNT_2D {
                        // Draw rays
                        let direction = Vector2::from_angle(
                            r as f64 * PI / (LIGHT_RAY_COUNT_2D as f64 / 2.0),
                            1.0,
                        );
                        let ray_start = light.pos.add(&direction.scale(3.0));
                        let segment = region.single_ray_trace(ray_start, direction);

                        draw_ray_segment_2d(canvas, &segment, colour, colour);
                    }
                }
                LightKind::PORTAL { portal_line: line } => {
                    draw_portal_light_2d(region, canvas, light, line);
                }
            }
        }

        // Draw walls
        for wall in region.walls() {
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
        let region = world.get_region(world.player().entity.region);
        let segments = world.ray_trace(region.id, world.player().entity.pos, look_direction);
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
    let end = world
        .player()
        .entity
        .pos
        .add(&world.player().look_direction.scale(half_player_size as f64));
    canvas.draw_between(world.player().entity.pos, end);
}

fn draw_portal_light_2d<R: RenderStrategy>(
    region: &Region,
    canvas: &mut R,
    light: &LightSource,
    portal_line: &LineSegment2,
) {
    canvas.set_draw_color(light.intensity.scale(0.2));
    for r in 0..LIGHT_RAY_COUNT_2D {
        let direction = Vector2::from_angle(r as f64 * PI / (LIGHT_RAY_COUNT_2D as f64 / 2.0), 1.0);

        let light_toward_portal = LineSegment2::from(light.pos, direction.scale(100.0));
        let point_on_portal = portal_line.algebraic_intersection(&light_toward_portal);
        let actually_crosses_portal = portal_line.contains(&point_on_portal);
        if !actually_crosses_portal {
            continue;
        }

        let direction = point_on_portal.subtract(&light.pos);
        let segment = region.single_ray_trace(point_on_portal.add(&direction.tiny()), direction);
        let line = region.trace_clear_portal_light(light, portal_line, segment.line.get_b());
        match line {
            None => {}
            Some(line) => {
                canvas.draw_between(line.a, line.b);
            }
        }
    }

    canvas.set_draw_color(light.intensity.scale(1.0));
    canvas.draw_between(portal_line.middle(), light.pos);
}

fn draw_wall_2d<R: RenderStrategy>(canvas: &mut R, wall: &Wall, contains_the_player: bool) {
    let color = if contains_the_player {
        match wall.portal() {
            Some { .. } => Colour::rgb(0, 255, 255),
            None => Colour::rgb(0, 255, 0),
        }
    } else {
        match wall.portal() {
            None => Colour::rgb(0, 0, 255),
            Some { .. } => Colour::rgb(0, 155, 15),
        }
    };

    canvas.set_draw_color(color);
    canvas.draw_between(wall.line().a, wall.line().b);

    // Draw normal
    canvas.set_draw_color(Colour::rgb(200, 0, 200));
    canvas.draw_between(
        wall.line().middle(),
        wall.line().middle().add(&wall.normal().scale(5.0)),
    );
}

fn draw_ray_segment_2d<R: RenderStrategy>(
    canvas: &mut R,
    segment: &RaySegment,
    hit_colour: Colour,
    miss_colour: Colour,
) {
    match segment.hit_wall {
        None => {
            canvas.set_draw_color(miss_colour);
            canvas.draw_between(
                segment.line.a,
                segment
                    .line
                    .a
                    .add(&segment.line.direction().normalize().scale(-100.0)),
            );
        }
        Some(_) => {
            canvas.set_draw_color(hit_colour);
            canvas.draw_between(segment.line.a, segment.line.b);
        }
    }
}
