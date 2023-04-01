use std::thread;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use crate::camera;
use crate::player::Player;
use crate::world::World;
use crate::world_gen::{example_preset, random_maze_world};

// TODO: calculate dynamically based on target FPS
const FRAME_DELAY_MS: u64 = 0;
const IDLE_FRAME_DELAY_MS: u64 = 20;

pub(crate) struct GameState {
    events: EventPump,
    world: World,
    start: Instant,
    seconds_counter: f64,
    render_frame_counter: i32,
    idle_frame_counter: i32,
    pause_seconds_counter: f64,
    total_delay_ms: u64,
    canvas: WindowCanvas,
}

impl GameState {
    pub(crate) fn new() -> Result<GameState, String> {
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

        Ok(GameState {
            events,
            world,
            start,
            seconds_counter,
            render_frame_counter,
            idle_frame_counter,
            pause_seconds_counter,
            total_delay_ms,
            canvas
        })
    }

    pub(crate) fn tick(&mut self) -> bool {
        let mut delta_mouse = 0;
        for event in self.events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return true,

                Event::KeyDown { keycode: Some(Keycode::Space), .. }
                => {
                    self.world.player_mut().first_person_rendering = !self.world.player().first_person_rendering;
                    *self.world.player_mut().needs_render_update.write().unwrap() = true;
                },

                Event::KeyDown { keycode: Some(Keycode::R), .. }
                => {
                    let player_pos = self.world.player().entity.pos;
                    let player_facing = self.world.player().look_direction;
                    self.world = random_maze_world();
                    *self.world.player_mut().needs_render_update.write().unwrap() = true;
                    self.world.player_mut().entity.pos = player_pos;
                    self.world.player_mut().look_direction = player_facing;
                },

                Event::MouseButtonDown { mouse_btn, .. } => {
                    Player::mouse_click(&mut self.world, mouse_btn);
                },

                Event::MouseMotion { xrel, .. } => {
                    delta_mouse += xrel;
                }
                _ => {}
            }
        }

        let keys = self.events.keyboard_state();
        let keys: Vec<Keycode> = keys.pressed_scancodes().filter_map(Keycode::from_scancode).collect();
        let duration = self.start.elapsed().as_secs_f64();

        self.seconds_counter += duration;

        if self.seconds_counter > 5.0 {
            let ms_counter = self.seconds_counter * 1000.0;
            let frame_time = (ms_counter / ((self.render_frame_counter + self.idle_frame_counter) as f64)).round() as u64;
            let fps = ((self.render_frame_counter + self.idle_frame_counter) as f64 / self.seconds_counter).round();
            // TODO: count render time seperatly so i can only count fps while it was actually rendering
            let sleep_percent = ((self.total_delay_ms as f64) / ms_counter * 100.0).round();
            let pause_percent = (self.pause_seconds_counter / self.seconds_counter * 100.0).round();
            println!("{} seconds; rendered {} frames; idle {} frames; {} fps; {} ms per frame (sleeping {}%, idle {}%)", self.seconds_counter.round(), self.render_frame_counter, self.idle_frame_counter, fps, frame_time, sleep_percent, pause_percent);
            self.seconds_counter = 0.0;
            self.render_frame_counter = 0;
            self.pause_seconds_counter = 0.0;
            self.total_delay_ms = 0;
            self.idle_frame_counter = 0;
        }

        self.start = Instant::now();

        self.world.update(duration, &keys, delta_mouse);

        // If you didn't move or turn and nothing in the world changed, don't bother redrawing the screen.
        // TODO: this needs to wait for lighting updates
        let needs_render_update = *self.world.player().needs_render_update.read().unwrap();
        let sleep_time = if needs_render_update {
            camera::render_scene(&mut self.canvas, &self.world, duration);
            self.render_frame_counter += 1;
            FRAME_DELAY_MS
        } else {
            self.pause_seconds_counter += duration;
            self.idle_frame_counter += 1;
            IDLE_FRAME_DELAY_MS
        };

        if sleep_time > 0 {
            // thread::sleep(std::time::Duration::from_millis(sleep_time));
            self.total_delay_ms += sleep_time;
        }

        false
    }
}
