use std::f64::consts::PI;
use std::fmt;

use sdl2::libc::c_int;
use sdl2::sys::SDL_Point;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64
}

const EPSILON: f64 = 0.000001;

impl Vector2 {
    pub(crate) const NAN: Vector2 = Vector2::of(f64::NAN, f64::NAN);

    pub fn new() -> Vector2 {
        Vector2 { x: 0.0, y: 0.0 }
    }

    pub const fn of(x: f64, y: f64) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Vector2 {
        let len = self.length();
        if len != 0.0 {
            Vector2::of(self.x / len, self.y / len)
        } else {
            Vector2::new()
        }
    }

    pub(crate) fn dot(&self, other: &Vector2) -> f64 {
        return (self.x * other.x) + (self.y * other.y);
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

    pub(crate) fn negate(&self) -> Vector2 {
        Vector2::of(0.0, 0.0).subtract(self)
    }

    pub(crate) fn from_angle(radians_from_origin: f64, length: f64) -> Vector2 {
        Vector2::of(radians_from_origin.cos() * length, radians_from_origin.sin() * length)
    }

    pub(crate) fn rotate(&self, delta_radians: f64) -> Vector2 {
        Vector2::from_angle(self.angle_from_origin() + delta_radians, self.length())
    }

    pub(crate) fn angle_from_origin(&self) -> f64 {
        if self.y >= 0.0 {
            self.normalize().x.acos()
        } else {
            self.normalize().x.acos() + PI
        }

    }

    pub(crate) fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    pub fn almost_equal(&self, other: &Vector2) -> bool {
        return (self.x - other.x).abs() < EPSILON && (self.y - other.y).abs() < EPSILON
    }

    pub fn sdl(&self) -> SDL_Point {
        SDL_Point {
            x: self.x as c_int,
            y: self.y as c_int,
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.length().abs() < EPSILON
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct LineSegment2 {
    pub(crate) a: Vector2,
    pub(crate) b: Vector2
}

impl LineSegment2 {
    pub(crate) fn of(start_point: Vector2, end_point: Vector2) -> LineSegment2 {
        LineSegment2 {a: start_point, b: end_point}
    }

    pub(crate) fn from(origin: Vector2, direction: Vector2) -> LineSegment2 {
        LineSegment2 {
            b: origin.add(&direction),
            a: origin
        }
    }

    pub(crate) fn algebraic(slope: f64, y_intercept: f64) -> LineSegment2 {
        LineSegment2 {
            a: Vector2::of(0.0, y_intercept),
            b: Vector2::of(1.0, y_intercept + slope),
        }
    }

    pub(crate) fn vertical(x: f64) -> LineSegment2 {
        LineSegment2::from(Vector2::of(x, 0.0), Vector2::of(0.0, 1.0))
    }

    pub(crate) fn horizontal(y: f64) -> LineSegment2 {
        LineSegment2::from(Vector2::of(0.0, y), Vector2::of(1.0, 0.0))
    }

    pub(crate) fn length(&self) -> f64 {
        self.direction().length()
    }

    pub(crate) fn direction(&self) -> Vector2 {
        self.a.subtract(&self.b)
    }

    pub(crate) fn slope(&self) -> f64 {
        self.direction().y / self.direction().x
    }

    pub(crate) fn is_horizontal(&self) -> bool {
        self.a.y == self.b.y
    }

    pub(crate) fn is_vertical(&self) -> bool {
        self.a.x == self.b.x
    }

    pub(crate) fn y_intercept(&self) -> f64 {
        if self.is_vertical() {
            return f64::NAN;
        }

        self.a.y - (self.a.x * self.slope())
    }

    pub(crate) fn direction_to(&self, point: &Vector2) -> Vector2 {
        self.closest_point(point).subtract(point)
    }

    /// Find the closest point on the algebraic line.
    pub(crate) fn closest_point(&self, point: &Vector2) -> Vector2 {
        if self.is_vertical() {
            return Vector2::of(self.a.x, point.y);
        }

        if self.is_horizontal() {
            return Vector2::of(point.x, self.a.y);
        }

        let goal_slope = 1.0 / self.slope();
        let new_line = LineSegment2 {
            a: self.a.clone(),
            b: Vector2::of(self.a.x + 1.0, self.a.y + goal_slope),
        };

        self.algebraic_intersection(&new_line)
    }

    /// Returns true if the algebraic lines intersect and that point in the range of both line segments.
    pub(crate) fn overlaps(&self, other: &LineSegment2) -> bool {
        !self.intersection(other).is_nan()
    }

    /// Returns true if the point is on the actual line segment (not just the algebraic line).
    /// Correctly returns false for nan points because any comparison against nan is false.
    pub(crate) fn contains(&self, point: &Vector2) -> bool {
        point.y >= self.a.y.min(self.b.y) && point.y <= self.a.y.max(self.b.y)
            && point.x >= self.a.x.min(self.b.x) && point.x <= self.a.x.max(self.b.x)
    }

    /// Returns NAN if the point is not in the range of both segments.
    pub(crate) fn intersection(&self, other: &LineSegment2) -> Vector2 {
        let hit = self.algebraic_intersection(other);

        if self.contains(&hit) && other.contains(&hit) {
            hit
        } else {
            Vector2::NAN
        }
    }

    /// The point might not actually be on the line segment, if the infinite algebraic line intersect but are far apart.
    /// Doesn't handle infinite points when they're the same line.
    pub(crate) fn algebraic_intersection(&self, other: &LineSegment2) -> Vector2 {
        let mut a = [-self.slope(), 1.0, self.y_intercept()];
        let mut b = [-other.slope(), 1.0, other.y_intercept()];

        if self.is_vertical(){
            a = [1.0, 0.0, self.a.x];
        }

        if other.is_vertical(){
            b = [1.0, 0.0, other.a.x];
        }


        reduce(&mut a, &mut b);
        Vector2::of(a[2], b[2])
    }
}


// insane hand rolled 2x2 row reduction cause i'm just experimenting with what lines are
pub fn reduce(r1: &mut [f64; 3], r2: &mut [f64; 3]) {
    if r2[0] != 0.0 && r1[0] == 0.0 {
        for i in 0..3 {
            let temp = r1[i];
            r1[i] = r2[i];
            r2[i] = temp;
        }
    }

    if (r1[0] == 0.0 && r2[0] == 0.0) || (r1[1] == 0.0 && r2[1] == 0.0) {
        r1[2] = f64::NAN;
        r2[2] = f64::NAN;
        return;
    }

    if r2[0] != 0.0 {
        let scale = r2[0] / r1[0];

        r2[0] = r2[0] - (scale * r1[0]);
        r2[1] = r2[1] - (scale * r1[1]);
        r2[2] = r2[2] - (scale * r1[2]);
    }

    if r1[1] != 0.0 {
        let scale = r1[1] / r2[1];

        r1[0] = r1[0] - (scale * r2[0]);
        r1[1] = r1[1] - (scale * r2[1]);
        r1[2] = r1[2] - (scale * r2[2]);
    }

    if r2[1] != 1.0 {
        r2[0] /= r2[1];
        r2[2] /= r2[1];
        r2[1] /= r2[1];
    }

    if r1[0] != 1.0 {
        r1[1] /= r1[0];
        r1[2] /= r1[0];
        r1[0] /= r1[0];
    }
}

impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_reduce() {
        assert_linear_system([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], -1.0, 2.0);
        assert_linear_system([5.0, 0.0, 3.0], [4.0, 2.0, 6.0], 0.6, 1.8);
        assert_linear_system([0.0, 2.0, 3.0], [0.0, 3.0, 6.0], f64::NAN, f64::NAN);
        assert_linear_system([-2.0, 1.0, 5.0], [0.0, 1.0, 3.0], -1.0, 3.0);
        assert_linear_system([0.0, 1.0, 3.0], [-2.0, 1.0, 5.0], -1.0, 3.0);
    }

    #[test]
    fn lines() {
        let line = LineSegment2::algebraic(2.0, 5.0);
        assert_eq!(line.slope(), 2.0);
        assert_eq!(line.y_intercept(), 5.0);

        let a = LineSegment2::algebraic(3.0, 2.0);
        let b = LineSegment2::algebraic(2.0, 3.0);
        assert_eq_vec(Vector2::of(1.0, 5.0), a.algebraic_intersection(&b));
        assert_eq_vec(Vector2::of(1.0, 5.0), b.algebraic_intersection(&a));

        let h = LineSegment2::horizontal(3.0);
        assert!(h.is_horizontal());
        assert_eq!(h.slope(), 0.0);
        assert_eq!(h.y_intercept(), 3.0);
        assert_intersect(line, h, -1.0, 3.0);

        let v = LineSegment2::vertical(-2.0);
        assert!(v.is_vertical());
        assert!(v.slope().is_infinite());
        assert!(v.y_intercept().is_nan());
        assert_intersect(line, v, -2.0, 1.0);
        assert_intersect(h, v, -2.0, 3.0);
    }

    fn assert_intersect(a: LineSegment2, b: LineSegment2, x: f64, y: f64){
        assert_eq_vec(a.algebraic_intersection(&b), Vector2::of(x, y));
        assert_eq_vec(b.algebraic_intersection(&a), Vector2::of(x, y));
    }

    fn assert_eq_vec(a: Vector2, b: Vector2){
        if !a.almost_equal(&b) {
            panic!("{:?} != {:?}", a, b);
        }
    }

    fn assert_linear_system(r1: [f64; 3], r2: [f64; 3], x: f64, y: f64){
        let mut r1_result = r1.clone();
        let mut r2_result = r2.clone();
        reduce(&mut r1_result, &mut r2_result);

        let x_success = (r1_result[2] - x).abs() < EPSILON || (r1_result[2].is_nan() && x.is_nan());
        let y_success = (r2_result[2] - y).abs() < EPSILON || (r2_result[2].is_nan() && y.is_nan());

        if !x_success || !y_success {
            panic!("\n{:?} -> {:?} \n{:?} -> {:?} \nExpected ({}, {}) but got ({}, {}).", r1, r1_result, r2, r2_result, x, y, r1_result[2], r2_result[2])
        }
    }
}

