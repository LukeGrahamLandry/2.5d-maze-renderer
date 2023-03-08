use std::collections::hash_set::IntoIter;
use std::collections::HashSet;
use std::slice::Iter;
use sdl2::keyboard::Keycode;

pub struct World {
    pub(crate) x: f64,
    pub(crate) y: f64
}

struct Vector2 {
    x: f64,
    y: f64
}

impl Vector2 {
    fn new() -> Vector2 {
        Vector2 { x: 0.0, y: 0.0 }
    }

    fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalize(&mut self){
        let len = self.length();
        if len != 0.0 {
            self.x /= len;
            self.y /= len;
        }
    }
}

impl World {
    pub(crate) fn new() -> World {
        World { x: 0.0, y: 0.0 }
    }

    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>){
        let speed = 200 as f64;

        let mut direction = Vector2::new();

        for key in pressed {
            match key {
                Keycode::W => {
                    direction.y = -1.0;
                }
                Keycode::S => {
                    direction.y = 1.0;
                }
                Keycode::A => {
                    direction.x = -1.0;
                }
                Keycode::D => {
                    direction.x = 1.0;
                }
                _ => (),
            }
        }

        direction.normalize();

        self.x += direction.x * speed * delta_time;
        self.y += direction.y * speed * delta_time;
    }
}