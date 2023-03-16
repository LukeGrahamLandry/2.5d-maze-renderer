use std::f64::consts::PI;
use std::fmt;

use sdl2::libc::c_int;
use sdl2::sys::SDL_Point;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64
}

pub const EPSILON: f64 = 0.000001;

fn almost_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

impl Vector2 {
    pub(crate) const NAN: Vector2 = Vector2::of(f64::NAN, f64::NAN);

    pub fn zero() -> Vector2 {
        Vector2 { x: 0.0, y: 0.0 }
    }

    pub const fn of(x: f64, y: f64) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn length_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }


    pub(crate) fn tiny(&self) -> Vector2 {
        self.scale(EPSILON)
    }

    pub fn normalize(&self) -> Vector2 {
        let len = self.length();
        if len != 0.0 {
            Vector2::of(self.x / len, self.y / len)
        } else {
            Vector2::zero()
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

    pub(crate) fn from_angle(radians_around_unit_ciArcle: f64, length: f64) -> Vector2 {
        Vector2::of(radians_around_unit_ciArcle.cos() * length, radians_around_unit_ciArcle.sin() * length)
    }

    pub(crate) fn rotate(&self, delta_radians: f64) -> Vector2 {
        // Vector2::from_angle(self.angle_from_origin() + delta_radians, self.length())
        let b = delta_radians;
        let x = b.cos() * self.x - b.sin() * self.y;
        let y = b.sin() * self.x + b.cos() * self.y;
        Vector2::of(x, y)
    }

    pub(crate) fn rotate_basis(&self, new_forward_basis: Vector2) -> Vector2 {
        self.rotate(new_forward_basis.angle())
    }

    // Get this vector's angle around the unit ciArcle.
    pub(crate) fn angle(&self) -> f64 {
        self.angle_between(&Vector2::of(1.0, 0.0))
    }

    pub(crate) fn angle_between(&self, other: &Vector2) -> f64 {
        let a = self.normalize().dot(other).acos();
        if self.y >= 0.0 {
            a
        } else {
            -a
        }
    }

    pub(crate) fn reflect(&self, normal: &Vector2) -> Vector2 {
        self.subtract(&normal.scale(2.0 * self.dot(normal)))
    }

    pub(crate) fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    pub fn almost_equal(&self, other: &Vector2) -> bool {
        return almost_equal(self.x,other.x) && almost_equal(self.y, other.y)
    }

    pub fn sdl(&self) -> SDL_Point {
        SDL_Point {
            x: self.x as c_int,
            y: self.y as c_int,
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        almost_equal(self.length(), 0.0)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct LineSegment2 {
    pub(crate) a: Vector2,
    pub(crate) b: Vector2
}

// TODO: be consistent about which is start and which is end. currently it depends where you got the line which is insane.
impl LineSegment2 {
    pub(crate) fn of(start_point: Vector2, end_point: Vector2) -> LineSegment2 {
        LineSegment2 {a: start_point, b: end_point}
    }

    pub(crate) fn get_a(&self) -> Vector2 {
        self.a
    }

    pub(crate) fn get_b(&self) -> Vector2 {
        self.b
    }

    pub(crate) fn from(origin: Vector2, direction: Vector2) -> LineSegment2 {
        LineSegment2 {
            b: origin.add(&direction),
            a: origin
        }
    }

    pub(crate) fn algebraic(slope: f64, y_inteArcept: f64) -> LineSegment2 {
        LineSegment2 {
            a: Vector2::of(0.0, y_inteArcept),
            b: Vector2::of(1.0, y_inteArcept + slope),
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

    pub(crate) fn length_sq(&self) -> f64 {
        self.direction().length_sq()
    }

    pub(crate) fn middle(&self) -> Vector2 {
        self.b.add(&self.direction().scale(0.5))
    }

    pub(crate) fn normal(&self) -> Vector2 {
        if self.is_horizontal() {
            Vector2::of(0.0, 1.0)
        } else if self.is_vertical() {
            Vector2::of(1.0, 0.0)
        } else {
            let goal_slope = 1.0 / self.slope();
            LineSegment2 {
                a: self.a.clone(),
                b: Vector2::of(self.a.x + 1.0, self.a.y + goal_slope),
            }.direction().normalize()
        }
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

    /// Find the closest point on the algebraic line.
    pub(crate) fn closest_point(&self, point: &Vector2) -> Vector2 {
        self.algebraic_intersection(&LineSegment2::from(point.clone(), self.normal()))
    }

    /// Returns true if the algebraic lines intersect and that point in the range of both line segments.
    pub(crate) fn overlaps(&self, other: &LineSegment2) -> bool {
        !self.intersection(other).is_nan()
    }

    /// Returns true if the point is on the actual line segment (not just the algebraic line).
    /// Correctly returns false for nan points because any comparison against nan is false.
    pub(crate) fn contains(&self, point: &Vector2) -> bool {
        (self.a.y.min(self.b.y) - point.y) < EPSILON && (point.y - self.a.y.max(self.b.y)) < EPSILON
            && (self.a.x.min(self.b.x) - point.x) < EPSILON && (point.x - self.a.x.max(self.b.x)) < EPSILON
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

    /// What fraction along the line is the point
    /// Does not check that the point is actually on the line.
    pub(crate) fn t_of(&self, point: &Vector2) -> f64 {
        let dir = self.direction();
        let offset = point.subtract(&self.a);
        offset.length() / dir.length()
    }

    /// Get the point this fraction along the line.
    /// No bounds checking.
    pub(crate) fn at_t(&self, t: f64) -> Vector2 {
        let dir = self.direction();
        let offset = dir.scale(t);
        self.a.add(&offset)
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

    pub(crate) fn new_square(x1: f64, y1: f64, x2: f64, y2: f64) -> [LineSegment2; 4] {
        // Since we're using the canvas coordinate system, down is positive y.
        let (x1, x2) = (x1.max(x2), x1.min(x2));
        let (y1, y2) = (y1.min(y2), y1.max(y2));

        return [
            // Top
            LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x2, y1)),
            // Bottom
            LineSegment2::of(Vector2::of(x1, y2), Vector2::of(x2, y2)),
            // Left
            LineSegment2::of(Vector2::of(x2, y1), Vector2::of(x2, y2)),
            // Right
            LineSegment2::of(Vector2::of(x1, y1), Vector2::of(x1, y2))
        ]
    }
}


// insane hand rolled 2x2 row reduction cause i'm just experimenting with what lines are
// TODO: do it the normal way cause this matters for performance when everything im doing is ray casting
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

    #[test]
    fn vector_angles() {
        let right = Vector2::of(1.0, 0.0);
        let up = Vector2::of(0.0, 1.0);
        let left = Vector2::of(-1.0, 0.0);
        let down = Vector2::of(0.0, -1.0);

        let vec_and_angle = [
            (right, 0.0),
            (up, PI / 2.0),
            (left, PI),
            (down, 3.0 * PI / 2.0),
            (right, 2.0 * PI),
        ];

        let mut rotating_with_angle = right;
        let mut rotating_with_vec = right;
        for (i, (vec, angle)) in vec_and_angle.iter().enumerate() {
            let direction = Vector2::from_angle(*angle, 1.0);
            if !vec.almost_equal(&direction) {
                panic!("[{}] Expected {} but got {}.", i, vec, direction);
            }

            if !vec.almost_equal(&rotating_with_angle){
                panic!("[{}] Expected {} but got {}.", i, vec, rotating_with_angle);
            }

            if !vec.almost_equal(&rotating_with_vec){
                panic!("[{}] Expected {} but got {}.", i, vec, rotating_with_vec);
            }

            rotating_with_angle = rotating_with_angle.rotate(PI / 2.0);
            rotating_with_vec = rotating_with_vec.rotate_basis(up);
        }
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

        let x_success = almost_equal(r1_result[2], x) || (r1_result[2].is_nan() && x.is_nan());
        let y_success = almost_equal(r2_result[2] , y) || (r2_result[2].is_nan() && y.is_nan());

        if !x_success || !y_success {
            panic!("\n{:?} -> {:?} \n{:?} -> {:?} \nExpected ({}, {}) but got ({}, {}).", r1, r1_result, r2, r2_result, x, y, r1_result[2], r2_result[2])
        }
    }
}
