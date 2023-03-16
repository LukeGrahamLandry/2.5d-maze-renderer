use std::cell::RefCell;
use std::f64::consts::PI;
use std::sync::{Arc, mpsc, RwLock, RwLockReadGuard};
use std::sync::mpsc::Sender;
use std::{panic, process, thread};
use sdl2::keyboard::Keycode::V;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::ray::{HitKind, HitResult, ray_trace, single_ray_trace, trace_clear_portal_light};
use crate::world_data::{ColumnLight, Player, Region, Wall, World};

const FOV_DEG: i32 = 45;
const SCREEN_HEIGHT: f64 = 600.0;
pub const SCREEN_WIDTH: u32 = 800;
const RESOLUTION_FACTOR: f64 = 1.0;
const LIGHT_RAY_COUNT_2D: i32 = 32;

// TODO: run length encoding for colours might be cool
struct ColouredLine {
    colour: Colour,
    line: LineSegment2
}

pub(crate) struct RenderBuffer<'a> {
    current_colour: Colour,
    offset: Vector2,
    sender: &'a mut dyn FnMut(ColouredLine),
}

impl<'a> RenderBuffer<'a> {
    fn new(sender: &mut dyn FnMut(ColouredLine)) -> RenderBuffer {
        RenderBuffer { sender, current_colour: Colour::black(), offset: Vector2::zero() }
    }

    fn set_draw_color(&mut self, colour: Colour){
        self.current_colour = colour;
    }

    fn draw_between(&mut self, start: Vector2, end: Vector2){
        self.draw_line(LineSegment2::of(start, end));
    }

    fn draw_line(&mut self, line: LineSegment2){
        let line = if self.offset.is_zero() {
            line
        } else {
            LineSegment2::of(line.get_a().add(&self.offset), line.get_b().add(&self.offset))
        };

        (self.sender)(ColouredLine { line, colour: self.current_colour });
    }
}

pub(crate) fn render2d(world: &World, window: &mut WindowCanvas, _delta_time: f64){
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
            window.draw_line(line.line.a.sdl(), line.line.b.sdl()).expect("SDL draw failed.");
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

fn draw_portal_light_2d(canvas: &mut RenderBuffer, wall: &Wall, light: &ColumnLight, fake_location: &LineSegment2, region: &Region) {
    let light_fake_origin = fake_location.get_b();
    let light_hits_portal_at = fake_location.get_a();

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

fn draw_ray_segment_2d(canvas: &mut RenderBuffer, segment: &HitResult, hit_colour: Colour, miss_colour: Colour) {
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

pub(crate) fn render3d(world: &World, window: &mut WindowCanvas, _delta_time: f64){
    let (sender, receiver) = mpsc::channel();

    let thread_count = 4 as usize;
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

                        let x = (x as f64 / RESOLUTION_FACTOR) as i32;
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
            for line in lines {
                window.set_draw_color(line.colour.to_u8());
                window.draw_line(line.line.a.sdl(), line.line.b.sdl()).expect("SDL draw failed.");
            }
        }
    });
}

pub(crate) fn render_column(world: &World, canvas: &mut RenderBuffer, x: i32){
    let look_direction = ray_direction_for_x(x, &world.player.peek().look_direction);
    let segments = ray_trace(world.player.peek().pos, look_direction, &world.player.peek().region.peek());

    let mut cumulative_dist = 0.0;
    for segment in &segments {
        draw_floor_segment(canvas, segment, x, cumulative_dist);
        cumulative_dist += segment.line.length();
    }

    draw_wall_3d(&world.player.peek(), canvas, segments.last().unwrap(), look_direction, cumulative_dist, x);
}

fn draw_floor_segment(canvas: &mut RenderBuffer, segment: &HitResult, screen_x: i32, cumulative_dist: f64){
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
            let colour = current.lerp(&next, t);  // TODO: its a quadratic not a line.
            canvas.set_draw_color(colour);
            canvas.draw_between(Vector2::of(screen_x as f64, bottom), Vector2::of(screen_x as f64, top));

            last_top = top;
        }
    }
}

// the sample_count should be high enough that we have one past the end to lerp to
fn light_floor_segment(segment: &HitResult, sample_length: f64, sample_count: i32) -> Vec<Colour> {
    let ray_line = segment.line;
    let mut samples: Vec<Colour> = Vec::with_capacity((sample_count) as usize);
    for i in 0..sample_count {
        let pos = ray_line.a.add(&ray_line.direction().normalize().scale(i as f64 * -sample_length));
        samples.push(light_floor_point(segment.region.peek(), pos));
    }

    samples
}

fn light_floor_point(region: &Region, hit_pos: Vector2) -> Colour {
    let mut colour = Colour::black();
    for light in &region.lights {
        colour = colour.add(region.floor_material.direct_floor_lighting(region, light.peek(), hit_pos));
    }
    for wall in region.iter_walls() {
        for light in wall.lights.iter() {
            colour = colour.add(region.floor_material.portal_floor_lighting(region, light.location, wall.line, light.parent.peek(), hit_pos));
        }
    }

    colour
}

fn draw_wall_3d(player: &Player, canvas: &mut RenderBuffer, hit: &HitResult, ray_direction: Vector2, cumulative_dist: f64, screen_x: i32) {
    let hit_point = hit.line.b;
    let wall_normal = match &hit.kind {
        HitKind::HitNone { .. } => { ray_direction }
        HitKind::HitWall { hit_wall, .. } => {
            hit_wall.peek().normal
        }
        HitKind::HitPlayer { box_side, .. } => {
            box_side.normal()
        }
    };

    let material = match &hit.kind {
        HitKind::HitNone { .. } => { Material::new(0.0, 0.0, 0.0) }
        HitKind::HitWall { hit_wall, .. } => {
            hit_wall.peek().material
        }
        HitKind::HitPlayer { .. } => {
            player.material
        }
    };

    let colour= light_wall_column(hit.region.peek(), &hit_point, wall_normal, &material, player, ray_direction, screen_x);
    let (top, bottom) = project_to_screen(cumulative_dist);

    canvas.set_draw_color(colour);
    canvas.draw_between(Vector2::of(screen_x as f64, top), Vector2::of(screen_x as f64, bottom));
}


fn light_wall_column(region: &Region, hit_point: &Vector2, wall_normal: Vector2, material: &Material, player: &Player, ray_direction: Vector2, x: i32) -> Colour {
    let middle = SCREEN_WIDTH as i32 / 2;

    let is_in_middle_half = (x - middle).abs() < (middle / 2);
    let has_flash_light = player.has_flash_light && is_in_middle_half;

    let mut colour = Colour::black();
    let to_eye = ray_direction.negate().normalize();
    for light in &region.lights {
        colour = colour.add(material.direct_wall_lighting(region, light.peek(), hit_point, wall_normal, &to_eye));
    }

    for wall in region.iter_walls() {
        for light in wall.lights.iter() {
            colour = colour.add(material.portal_wall_lighting(region, light.location, wall.line, light.parent.peek(), hit_point, wall_normal, &to_eye));
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
