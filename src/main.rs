extern crate sdl2;

use std::thread;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use crate::mth::Vector2;

use crate::world::World;

mod world;
mod player;
mod camera;
mod mth;

pub fn run() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("walls", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;

    let mut world = World::create_example();
    let mut first_person_rendering = false;

    canvas.clear();
    canvas.present();

    let mut events = sdl_context.event_pump()?;

    let mut start = Instant::now();

    'mainloop: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'mainloop,

                Event::KeyDown { keycode: Some(Keycode::Space), .. }
                => first_person_rendering = !first_person_rendering,

                Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                    world.on_mouse_click(mouse_btn);
                }
                _ => {}
            }
        }

        let keys =  events.keyboard_state();
        let keys: Vec<Keycode> = keys.pressed_scancodes().filter_map(Keycode::from_scancode).collect();
        let duration = start.elapsed().as_secs_f64();
        start = Instant::now();

        let sleep_time = std::time::Duration::from_millis(40);
        thread::sleep(sleep_time);

        let mouse_pos = Vector2::of(events.mouse_state().x() as f64, events.mouse_state().y() as f64);

        world.update(duration, &keys);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        if first_person_rendering {
            camera::render3d(&world, &mut canvas, duration, &mouse_pos);
        } else {
            camera::render2d(&world, &mut canvas, duration);
        }

        canvas.present();
    }

    Ok(())
}

fn main() -> Result<(), String> {
    run()?;

    Ok(())
}
