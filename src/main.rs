extern crate sdl2;
extern crate core;

use std::thread;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use world_gen::shift_the_world;
use crate::world_gen::random_maze_world;

mod world;
mod player;
mod camera3d;
mod mth;
mod world_gen;
mod ray;
mod material;
mod world_data;
mod shelf;
mod camera2d;
mod camera;
mod map_builder;
mod light_cache;
mod new_world;
mod lighting;

// TODO: calculate dynamically based on target FPS
const FRAME_DELAY_MS: u64 = 0;
const IDLE_FRAME_DELAY_MS: u64 = 20;

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

    canvas.clear();
    canvas.present();

    let mut events = sdl_context.event_pump()?;

    let mut start = Instant::now();

    let mut seconds_counter = 0.0;
    let mut render_frame_counter = 0;
    let mut idle_frame_counter = 0;
    let mut pause_seconds_counter = 0.0;
    let mut total_delay_ms = 0u64;
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
                    world.player.borrow_mut().first_person_rendering = !world.player.borrow_mut().first_person_rendering;
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

        if seconds_counter > 5.0 {
            let ms_counter = seconds_counter * 1000.0;
            let frame_time = (ms_counter / ((render_frame_counter + idle_frame_counter) as f64)).round() as u64;
            let fps = ((render_frame_counter + idle_frame_counter) as f64 / seconds_counter).round();
            // TODO: count render time seperatly so i can only count fps while it was actually rendering
            let sleep_percent = ((total_delay_ms as f64) / ms_counter * 100.0).round();
            let pause_percent = (pause_seconds_counter / seconds_counter * 100.0).round();
            println!("{} seconds; rendered {} frames; idle {} frames; {} fps; {} ms per frame (sleeping {}%, idle {}%)", seconds_counter.round(), render_frame_counter, idle_frame_counter, fps, frame_time, sleep_percent, pause_percent);
            seconds_counter = 0.0;
            render_frame_counter = 0;
            pause_seconds_counter = 0.0;
            total_delay_ms = 0;
            idle_frame_counter = 0;
        }

        start = Instant::now();

        world.update(duration, &keys, delta_mouse);

        // If you didn't move or turn and nothing in the world changed, don't bother redrawing the screen.
        let needs_render_update = *world.player.borrow().needs_render_update.read().unwrap();
        let sleep_time = if needs_render_update {
            camera::render_scene(&mut canvas, &world, duration);
            render_frame_counter += 1;
            FRAME_DELAY_MS
        } else {
            pause_seconds_counter += duration;
            idle_frame_counter += 1;
            IDLE_FRAME_DELAY_MS
        };
        
        if sleep_time > 0 {
            thread::sleep(std::time::Duration::from_millis(sleep_time));
            total_delay_ms += sleep_time;
        }   
    }

    Ok(())
}

fn main() -> Result<(), String> {
    run()?;

    Ok(())
}
