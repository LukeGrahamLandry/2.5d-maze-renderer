use crate::material::Colour;
use crate::mth::{LineSegment2, Vector2};
use crate::world::World;
use crate::{camera2d, camera3d};
use std::f64::consts::PI;
use softbuffer::Buffer;

pub const FOV_DEG: i32 = 45;
pub const SCREEN_HEIGHT: f64 = 600.0;
pub const SCREEN_WIDTH: u32 = 800;
pub const RESOLUTION_FACTOR: f64 = 1.0;
pub const LIGHT_RAY_COUNT_2D: i32 = 32;

pub(crate) fn render_scene<R: RenderStrategy>(canvas: &mut R, world: &World) {
    if world.player().first_person_rendering {
        camera3d::render(world, canvas);
    } else {
        camera2d::render(world, canvas);
    }

    *world.player().needs_render_update.write().unwrap() = false;
}

/// Converts a (distance to a wall) into a top and bottom y to draw that wall on the canvas.
/// https://nicolbolas.github.io/oldtut/Positioning/Tut04%20Perspective%20Projection.html
pub(crate) fn project_to_screen(z_distance: f64) -> (f64, f64) {
    let zoom_amount = 12000.0;
    let screen_wall_height = zoom_amount / z_distance;
    let screen_middle = SCREEN_HEIGHT / 2.0;
    let half_screen_wall_height = screen_wall_height / 2.0;
    let y_top = (screen_middle - half_screen_wall_height).max(0.0);
    let y_bottom = (screen_middle + half_screen_wall_height).min(SCREEN_HEIGHT - 1.0);
    (y_top, y_bottom)
}

pub(crate) fn x_to_angle(screen_x: i32) -> f64 {
    let t = screen_x as f64 / SCREEN_WIDTH as f64;
    let delta_deg = (t - 0.5) * FOV_DEG as f64;
    let delta_rad = PI * delta_deg / 180.0;
    delta_rad
}

/// Assuming the forwards points at the middle of the screen, return it rotated to point at x instead.
pub(crate) fn ray_direction_for_x(screen_x: i32, forwards: &Vector2) -> Vector2 {
    forwards.rotate(x_to_angle(screen_x))
}

// TODO: run length encoding for colours might be cool
pub(crate) struct ColouredLine {
    pub(crate) colour: Colour,
    pub(crate) a: Vector2,
    pub(crate) b: Vector2,
}

pub(crate) trait RenderStrategy {
    fn set_draw_color(&mut self, colour: Colour);
    fn draw_between(&mut self, start: Vector2, end: Vector2);
    fn draw_line(&mut self, line: LineSegment2);
}

pub struct SoftBufferRender<'a> {
    pub(crate) colour: Colour,
    pub(crate) buffer: Buffer<'a>,
    pub width: usize,
    pub height: usize
}

impl<'a> RenderStrategy for SoftBufferRender<'a> {
    fn set_draw_color(&mut self, colour: Colour) {
        self.colour = colour;
    }

    fn draw_between(&mut self, start: Vector2, end: Vector2) {
        let x = start.x as usize;
        if x >= self.width {
            return;
        }

        let y1 = start.y.min(end.y) as usize;
        let y2 = start.y.max(end.y) as usize;

        if y1 == y2 {
            let index = (y1 * self.width) + x;
            self.buffer[index] = self.colour.to_packed();
        } else {
            for y in y1.min(self.height)..y2.min(self.height) {
                let index = (y * self.width) + x;
                self.buffer[index] = self.colour.to_packed();
            }
        }


    }

    fn draw_line(&mut self, line: LineSegment2) {
        self.draw_between(line.a, line.b);
    }
}
