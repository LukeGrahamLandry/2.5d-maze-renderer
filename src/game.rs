use std::thread;
use std::time::Instant;

use crate::camera;
use crate::player::Player;
use crate::world::World;
use crate::world_gen::random_maze_world;

// TODO: calculate dynamically based on target FPS
const FRAME_DELAY_MS: u64 = 0;
const IDLE_FRAME_DELAY_MS: u64 = 20;

pub(crate) struct GameState {
    pub(crate) world: World,
    start: Instant,
    seconds_counter: f64,
    pub(crate) render_frame_counter: i32,
    idle_frame_counter: i32,
    pause_seconds_counter: f64,
    total_delay_ms: u64,
    pub(crate) delta_mouse: f32,
    pub keys: Keys,
}

impl GameState {
    pub(crate) fn new() -> GameState {
        let world = random_maze_world();

        GameState {
            world,
            start: Instant::now(),
            seconds_counter: 0.0,
            render_frame_counter: 0,
            idle_frame_counter: 0,
            pause_seconds_counter: 0.0,
            total_delay_ms: 0,
            delta_mouse: 0.0,
            keys: Keys::empty(),
        }
    }

    pub(crate) fn tick(&mut self) -> bool {
        let duration = self.start.elapsed().as_secs_f64();

        self.seconds_counter += duration;

        if self.seconds_counter > 5.0 {
            let render_seconds = self.seconds_counter - self.pause_seconds_counter;
            let frame_time =
                (render_seconds / (self.render_frame_counter as f64) * 1000.0).round() as u64;
            let fps = (self.render_frame_counter as f64 / render_seconds).round();
            println!(
                "{} seconds; rendered {} frames; {} fps; {} ms per frame",
                self.seconds_counter.round(),
                self.render_frame_counter,
                fps,
                frame_time
            );
            self.seconds_counter = 0.0;
            self.render_frame_counter = 0;
            self.pause_seconds_counter = 0.0;
            self.total_delay_ms = 0;
            self.idle_frame_counter = 0;
        }

        self.start = Instant::now();
        self.world.update(duration, &self.keys, self.delta_mouse as i32);
        self.delta_mouse = 0.0;

        // If you didn't move or turn and nothing in the world changed, don't bother redrawing the screen.
        // TODO: this needs to wait for lighting updates
        *self.world.player().needs_render_update.read().unwrap()
    }

    pub fn reset_world(&mut self) {
        let player_pos = self.world.player().entity.pos;
        let player_facing = self.world.player().look_direction;
        self.world = random_maze_world();
        *self.world.player_mut().needs_render_update.write().unwrap() = true;
        self.world.player_mut().entity.pos = player_pos;
        self.world.player_mut().look_direction = player_facing;
    }

    pub fn toggle_camera_mode(&mut self) {
        self.world.player_mut().first_person_rendering =
            !self.world.player().first_person_rendering;
        *self.world.player_mut().needs_render_update.write().unwrap() = true;
    }
}

pub struct Keys {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub f: bool,
}

impl Keys {
    fn empty() -> Keys {
        Keys {
            w: false,
            a: false,
            s: false,
            d: false,
            f: false,
        }
    }
}
