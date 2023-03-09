use std::fmt;
use sdl2::libc::c_int;
use sdl2::sys::SDL_Point;

pub struct Vector2 {
    pub x: f64,
    pub y: f64
}

impl Vector2 {
    pub(crate) fn copy(other: &Vector2) -> Vector2 {
        Vector2::of(other.x, other.y)
    }
}

impl Vector2 {
    pub fn new() -> Vector2 {
        Vector2 { x: 0.0, y: 0.0 }
    }

    pub fn of(x: f64, y: f64) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&mut self){
        let len = self.length();
        if len != 0.0 {
            self.x /= len;
            self.y /= len;
        }
    }

    pub(crate) fn subtract(&self, other: &Vector2) -> Vector2 {
        Vector2::of(self.x - other.x, self.y - other.y)
    }

    pub(crate) fn add(&self, other: &Vector2) -> Vector2 {
        Vector2::of(self.x + other.x, self.y + other.y)
    }

    pub(crate) fn scale(&self, s: f64) -> Vector2 {
        Vector2::of(self.x * s, self.y * s)
    }

    pub fn sdl(&self) -> SDL_Point {
        SDL_Point {
            x: self.x as c_int,
            y: self.y as c_int,
        }
    }
}


impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}