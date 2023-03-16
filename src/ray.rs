
use crate::mth::{EPSILON, LineSegment2, Vector2};
use crate::shelf::{ShelfPtr};
use crate::world_data::{Region, Wall, WorldThing};

const PORTAL_LIMIT: u16 = 15;
pub const VIEW_DIST: f64 = 1000.0;

/// Sends a ray through the world, following portals, and returns a separate line segment for each region it passes through.
pub(crate) fn ray_trace(mut origin: Vector2, mut direction: Vector2, region: &Region) -> Vec<HitResult> {
    let mut segments = vec![];

    let mut segment = single_ray_trace(origin, direction, region);
    for _ in 0..PORTAL_LIMIT {
        match &segment.kind.clone() {
            HitKind::HitNone { .. }
            | HitKind::HitPlayer { .. } => { break; }
            HitKind::HitWall { hit_wall, .. } => {
                let wall = hit_wall.borrow();

                match wall.get_next_wall() {
                    None => { break; }
                    Some(new_wall) => {
                        let t = wall.line.t_of(&segment.line.b).abs();
                        let hit_back = wall.normal.dot(&direction) > 0.0;
                        let hit_edge = t < 0.01 || t > 0.99;
                        if hit_back || hit_edge {
                            break;
                        }

                        // Go through the portal
                        let new_wall = new_wall.borrow();
                        origin = Wall::translate(segment.line.b, &wall, &new_wall);
                        direction = Wall::rotate(direction, &wall, &new_wall);

                        segments.push(segment.clone());
                        let region = new_wall.region.clone();
                        segment = single_ray_trace(origin.add(&direction), direction, &region.borrow());
                    }
                }
            }
        }
    }

    segments.push(segment);
    segments
}

pub(crate) fn trace_clear_path_between(origin: Vector2, target: Vector2, region: &Region) -> Option<Vec<HitResult>> {
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
pub(crate) fn trace_clear_path_no_portals_between(origin: Vector2, target: Vector2, region: &Region) -> Option<HitResult> {
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
pub(crate) fn trace_clear_portal_light(relative_light: LineSegment2, portal_wall: LineSegment2, target: Vector2, region: &Region) -> Option<LineSegment2> {
    let light_fake_origin = relative_light.get_b();
    let _pos_on_portal_closest_to_fake_light = relative_light.get_a();  // dont care

    let ray = LineSegment2::of(light_fake_origin, target);

    let ray_hit_portal_pos = ray.intersection(&portal_wall);
    if ray_hit_portal_pos.is_nan() {  // The light does not pass through the portal to the point.
        return None;
    }

    let direction = target.subtract(&ray_hit_portal_pos);

    let ray = LineSegment2::from(ray_hit_portal_pos.add(&direction.tiny()), direction.scale(1.0 - (5.0 * EPSILON)));
    for wall in region.iter_walls() {
        let hit = wall.line.intersection(&ray);
        if !hit.is_nan() {
            return None;
        }
    }

    Some(ray)
}

/// Sends a ray through a single region until it hits a wall. Without following portals.
pub(crate) fn single_ray_trace(origin: Vector2, direction: Vector2, region: &Region) -> HitResult {
    let ray = LineSegment2::from(origin, direction.scale(VIEW_DIST));

    let mut shortest_hit_distance_squared = f64::INFINITY;
    let mut closest_hit_point = Vector2::NAN;
    let mut hit_wall = None;

    for wall in region.iter_walls() {
        let hit = wall.line.intersection(&ray);
        let to_hit = origin.subtract(&hit);

        if !hit.is_nan() && to_hit.length_sq() < shortest_hit_distance_squared {
            hit_wall = Some(wall);
            shortest_hit_distance_squared = to_hit.length_sq();
            closest_hit_point = hit;
        }
    }

    let mut hit_result = match hit_wall {
        None => {
            HitResult {
                region: region.myself.clone(),
                line: LineSegment2::of(origin, origin.add(&direction.scale(VIEW_DIST))),
                kind: HitKind::HitNone
            }
        }
        Some(hit_wall) => {
            HitResult {
                region: region.myself.clone(),
                line: LineSegment2::of(origin, closest_hit_point),
                kind: HitKind::HitWall {
                    hit_wall: hit_wall.myself.clone()
                }
            }
        }
    };

    for (_id, thing) in &region.things {
        let hit = thing.borrow().collide(origin, direction);
        if hit.dist_squared() < hit_result.dist_squared() {
            hit_result = hit;
        }
    }

    hit_result
}

#[derive(Clone)]
pub struct HitResult {
    pub(crate) region: ShelfPtr<Region>,
    pub(crate) line: LineSegment2,
    pub(crate) kind: HitKind
}

impl HitResult {
    pub(crate) fn empty(region: ShelfPtr<Region>, origin: Vector2, direction: Vector2) -> HitResult {
        HitResult  {
            region,
            line: LineSegment2::of(origin, origin.add(&direction)),
            kind: HitKind::HitNone
        }
    }

    fn dist_squared(&self) -> f64 {
        match self.kind {
            HitKind::HitNone => { f64::INFINITY }
            HitKind::HitWall { .. } | HitKind::HitPlayer { .. } => { self.line.length_sq() }
        }
    }
}

#[derive(Clone)]
pub(crate) enum HitKind {
    HitNone,
    HitWall { hit_wall: ShelfPtr<Wall> },
    HitPlayer { box_side: LineSegment2 }
}
