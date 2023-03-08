use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use crate::world::World;

pub(crate) fn render(world: &World, canvas: &mut WindowCanvas, delta_time: f64){
    let x = world.x as i32;
    let y = world.y as i32;
    canvas.fill_rect(Rect::new(x, y, 10, 10)).expect("TODO: panic message");
}
