
extern crate sdl2;

use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadSurface};
use sdl2::keyboard::Keycode;
use sdl2::mouse::Cursor;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use std::{env, thread};
use std::path::Path;
use std::time::Instant;

pub fn run(png: &Path) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("doomthing", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;

    canvas.clear();
    canvas.present();

    canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));

    let mut events = sdl_context.event_pump()?;

    let mut start = Instant::now();

    'mainloop: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,

                Event::MouseButtonDown { x, y, .. } => {
                    println!("Click ({}, {})", x, y);
                    canvas.fill_rect(Rect::new(x, y, 10, 10))?;
                    canvas.present();
                }
                _ => {}
            }
        }

        let keys =  events.keyboard_state();
        let keys = keys.pressed_scancodes().filter_map(Keycode::from_scancode);
        for x in keys {
            println!("Key {}", x);
        }

        let duration = start.elapsed();
        start = Instant::now();
        println!("Frame time {:?}", duration);

        let sleep_time = std::time::Duration::from_millis(10);
        thread::sleep(sleep_time);
    }

    Ok(())
}

fn main() -> Result<(), String> {
    run(Path::new("/Users/luke/Documents/40009893.png"))?;
    Ok(())
}

