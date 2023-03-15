extern crate sdl2;

use std::thread;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use crate::maze_world::{random_maze_world, shift_the_world};
use crate::world::World;

mod world;
mod player;
mod camera;
mod mth;
mod maze_world;
mod ray;
mod material;


pub fn run() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    sdl_context.mouse().show_cursor(false);
    sdl_context.mouse().capture(true);
    sdl_context.mouse().set_relative_mouse_mode(true);

    let window = video_subsystem
        .window("walls", 800, 600)
        .position_centered()
        .input_grabbed()
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
        let mut delta_mouse = 0;
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'mainloop,

                Event::KeyDown { keycode: Some(Keycode::Space), .. }
                => first_person_rendering = !first_person_rendering,

                Event::KeyDown { keycode: Some(Keycode::R), .. }
                => shift_the_world(&mut world),

                Event::MouseButtonDown { mouse_btn, .. } => {
                    world.on_mouse_click(mouse_btn);
                },

                Event::MouseMotion { xrel, .. } => {
                    delta_mouse += xrel;
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

        world.update(duration, &keys, delta_mouse);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        if first_person_rendering {
            camera::render3d(&world, &mut canvas, duration);
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
