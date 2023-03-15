use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::mth::{EPSILON, LineSegment2, Vector2};
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

pub(crate) fn trace_clear_path_between(origin: Vector2, target: Vector2, region: &Rc<RefCell<Region>>) -> Option<Vec<HitResult>> {
    let direction = target.subtract(&origin).normalize();
    let segments = ray_trace(origin, direction, region);
    let last_hit = segments.last().unwrap();
    let has_clear_path = last_hit.line.b.almost_equal(&target);
    if has_clear_path {
        Some(segments)
    } else {
        None
    }
}

// This could use trace_clear_path_between and check that the vec is only one long but that would waste time tracing through portals that we don't care about.
pub(crate) fn trace_clear_path_no_portals_between(origin: Vector2, target: Vector2, region: &Rc<RefCell<Region>>) -> Option<HitResult> {
    let direction = target.subtract(&origin).normalize();
    let last_hit = single_ray_trace(origin, direction, region);
    let has_clear_path = last_hit.line.b.almost_equal(&target);
    if has_clear_path {
        Some(last_hit)
    } else {
        None
    }
}

// TODO: see if i can organize these better so internally there's one generic implementation with different flags
//       but also make sure it doesn't devolve into new RayCasterManagerAbstractFactoryBuilder().build().run(...)

// does not go through portals
/// Sends a ray through a single region without following portals. The ray starts from the far end of the relative_light line.
/// Returns the ray line segment (from portal to target) if it did not hit any walls after the portal and went through the portal.
pub(crate) fn trace_clear_portal_light(relative_light: LineSegment2, portal_wall: LineSegment2, target: Vector2, region: &Rc<RefCell<Region>>) -> Option<LineSegment2> {
    let light_fake_origin = relative_light.get_b();
    let _pos_on_portal_closest_to_fake_light = relative_light.get_a();  // dont care

    let ray = LineSegment2::of(light_fake_origin, target);

    let ray_hit_portal_pos = ray.intersection(&portal_wall);
    if ray_hit_portal_pos.is_nan() {  // The light does not pass through the portal to the point.
        return None;
    }

    let direction = target.subtract(&ray_hit_portal_pos);

    let ray = LineSegment2::from(ray_hit_portal_pos.add(&direction.tiny()), direction.scale(1.0 - (5.0 * EPSILON)));
    let m_region = region.borrow();
    for wall in &m_region.walls {
        let hit = wall.borrow().line.intersection(&ray);
        if !hit.is_nan() {
            return None;
        }
    }

    Some(ray)
}

/// Sends a ray through a single region until it hits a wall. Without following portals.
pub(crate) fn single_ray_trace(origin: Vector2, direction: Vector2, region: &Rc<RefCell<Region>>) -> HitResult {
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
    pub(crate) region: Weak<RefCell<Region>>,  // TODO: shouldn't be weak
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
