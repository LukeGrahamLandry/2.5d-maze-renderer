use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use crate::mth::Vector2;
use crate::world::World;

pub(crate) fn render(world: &World, canvas: &mut WindowCanvas, _delta_time: f64){
    let half_player_size = 5;
    let x = world.player.pos.x as i32;
    let y = world.player.pos.y as i32;

    canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
    canvas.fill_rect(Rect::new(x - half_player_size, y - half_player_size, (half_player_size * 2) as u32, (half_player_size * 2) as u32)).expect("Draw failed");

    for region in world.regions.iter() {
        for wall in region.walls.iter() {
            canvas.draw_line(wall.a.sdl(), wall.b.sdl()).expect("Draw failed");
        }
    }

    canvas.set_draw_color(Color::RGBA(255, 0, 0, 255));
    let player_view_end = world.player.pos.add(&world.player.direction.scale((half_player_size * 2) as f64));
    canvas.draw_line(world.player.pos.sdl(), player_view_end.sdl()).expect("Draw failed");


    canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
}


