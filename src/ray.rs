use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::mth::{LineSegment2, Vector2};
use crate::world::{Region, Wall};

const PORTAL_LIMIT: u16 = 15;
pub const VIEW_DIST: f64 = 1000.0;

/// Sends a ray through the world, following portals, and returns a separate line segment for each region it passes through.
pub(crate) fn ray_trace(mut origin: Vector2, mut direction: Vector2, region: &Rc<RefCell<Region>>) -> Vec<HitResult> {
    let mut segments = vec![];

    let mut segment = single_ray_trace(origin, direction, region);
    for _ in 0..PORTAL_LIMIT {
        match &segment.kind {
            HitKind::None { .. }
            | HitKind::Player { .. } => { break; }
            HitKind::Wall { hit_wall, .. } => {
                let wall = hit_wall.upgrade().unwrap();
                let wall = wall.borrow();

                match &wall.next_wall {
                    None => { break; }
                    Some(new_wall) => {
                        let t = wall.line.t_of(&segment.line.b).abs();
                        let hit_back = wall.normal.dot(&direction) > 0.0;
                        let hit_edge = t < 0.01 || t > 0.99;
                        if hit_back || hit_edge {
                            break;
                        }

                        // Go through the portal
                        let new_wall = new_wall.upgrade().unwrap();
                        let new_wall = new_wall.borrow();
                        origin = Wall::translate(segment.line.b, &wall, &new_wall);
                        direction = Wall::rotate(direction, &wall, &new_wall);

                        segments.push(segment);
                        segment = single_ray_trace(origin.add(&direction), direction, &new_wall.region.upgrade().unwrap());
                    }
                }
            }
        }
    }

    segments.push(segment);
    segments
}

/// Sends a ray through a single region until it hits a wall.
fn single_ray_trace(origin: Vector2, direction: Vector2, region: &Rc<RefCell<Region>>) -> HitResult {
    let ray = LineSegment2::from(origin, direction.scale(VIEW_DIST));

    let mut shortest_hit_distance = f64::INFINITY;
    let mut closest_hit_point = Vector2::NAN;
    let mut hit_wall = None;

    let m_region = region.borrow();
    for wall in &m_region.walls {
        let hit = wall.borrow().line.intersection(&ray);
        let to_hit = origin.subtract(&hit);

        if !hit.is_nan() && to_hit.length() < shortest_hit_distance {
            hit_wall = Some(wall);
            shortest_hit_distance = to_hit.length();
            closest_hit_point = hit;
        }
    }

    let mut hit_result = match hit_wall {
        None => {
            HitResult {
                region: Rc::downgrade(region),
                line: LineSegment2::of(origin, origin.add(&direction.scale(VIEW_DIST))),
                kind: HitKind::None
            }
        }
        Some(hit_wall) => {
            HitResult {
                region: Rc::downgrade(region),
                line: LineSegment2::of(origin, closest_hit_point),
                kind: HitKind::Wall {
                    hit_wall: Rc::downgrade(&hit_wall)
                }
            }
        }
    };

    for (_id, thing) in &m_region.things {
        let hit = thing.upgrade().unwrap().borrow().collide(origin, direction);
        if hit.dist() < hit_result.dist() {
            hit_result = hit;
        }
    }

    hit_result
}

#[derive(Debug, Clone)]
pub(crate) struct HitResult {
    pub(crate) region: Weak<RefCell<Region>>,
    pub(crate) line: LineSegment2,
    pub(crate) kind: HitKind
}

impl HitResult {
    pub(crate) fn empty(region: &Rc<RefCell<Region>>, origin: Vector2, direction: Vector2) -> HitResult {
        HitResult  {
            region: Rc::downgrade(region),
            line: LineSegment2::of(origin, origin.add(&direction)),
            kind: HitKind::None
        }
    }

    fn dist(&self) -> f64 {
        match self.kind {
            HitKind::None => { f64::INFINITY }
            HitKind::Wall { .. } | HitKind::Player { .. } => { self.line.length() }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum HitKind {
    None,
    Wall { hit_wall: Weak<RefCell<Wall>> },
    Player { box_side: LineSegment2 }
}
