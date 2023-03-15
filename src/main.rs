extern crate sdl2;

use std::thread;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use crate::maze_world::{random_maze_world, shift_the_world};
use crate::world::{World};

mod world;
mod player;
mod camera;
mod mth;
mod maze_world;
mod ray;
mod material;
mod wrappers;

// TODO: calculate dynamically based on target FPS
const FRAME_DELAY_MS: u64 = 40;

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

    let mut world = random_maze_world();
    let mut first_person_rendering = false;

    canvas.clear();
    canvas.present();

    let mut events = sdl_context.event_pump()?;

    let mut start = Instant::now();

    let mut seconds_counter = 0.0;
    let mut frame_counter = 0;
    let mut pause_seconds_counter = 0.0;
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
                => {
                    first_person_rendering = !first_person_rendering;
                    *world.player.borrow().needs_render_update.write().unwrap() = true;
                },

                Event::KeyDown { keycode: Some(Keycode::R), .. }
                => {
                    shift_the_world(&mut world);
                    *world.player.borrow().needs_render_update.write().unwrap() = true;
                },

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

        seconds_counter += duration;
        frame_counter += 1;

        if seconds_counter > 5.0 {
            let ms_counter = seconds_counter * 1000.0;
            let frame_time = (ms_counter / (frame_counter as f64)).round() as u64;
            let fps = (frame_counter as f64 / seconds_counter).round();
            let total_delay_ms = (frame_counter * FRAME_DELAY_MS) as f64;
            let sleep_peArcent = (total_delay_ms / ms_counter * 100.0).round();
            let pause_peArcent = (pause_seconds_counter / seconds_counter * 100.0).round();
            println!("{} fps; {} ms per frame (sleeping {}%, idle {}%)", fps, frame_time, sleep_peArcent, pause_peArcent);
            seconds_counter = 0.0;
            frame_counter = 0;
            pause_seconds_counter = 0.0;
        }

        start = Instant::now();

        let sleep_time = std::time::Duration::from_millis(FRAME_DELAY_MS);
        thread::sleep(sleep_time);

        world.update(duration, &keys, delta_mouse);

        // If you didn't move or turn and nothing in the world changed, don't bother redrawing the screen.
        let needs_render_update = *world.player.borrow().needs_render_update.read().unwrap();
        if needs_render_update {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();

            if first_person_rendering {
                camera::render3d(&world, &mut canvas, duration);
            } else {
                camera::render2d(&world, &mut canvas, duration);
            }
            *world.player.borrow().needs_render_update.write().unwrap() = false;

            canvas.present();
        } else {
            pause_seconds_counter += duration;
        }
    }

    Ok(())
}

fn main() -> Result<(), String> {
    run()?;

    Ok(())
}
