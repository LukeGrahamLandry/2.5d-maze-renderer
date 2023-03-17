use crate::light_cache::PortalLight;
use crate::map_builder::{MapRegion, MapWall};
use crate::mth::{EPSILON, LineSegment2, Vector2};

const PORTAL_LIMIT: u16 = 15;
pub const VIEW_DIST: f64 = 1000.0;

/// Sends a ray through the world, following portals, and returns a separate line segment for each region it passes through.
pub(crate) fn ray_trace<'a>(mut origin: Vector2, mut direction: Vector2, region: &'a MapRegion<'a>) -> Vec<HitResult<'a>> {
    let mut segments = vec![];

    let mut segment = single_ray_trace(origin, direction, region);
    for _ in 0..PORTAL_LIMIT {
        match &segment.kind.clone() {
            HitKind::HitNone { .. }
            | HitKind::HitPlayer { .. } => { break; }
            HitKind::HitWall { hit_wall, .. } => {
                match hit_wall.next_wall {
                    None => { break; }
                    Some(new_wall) => {
                        let t = hit_wall.line.t_of(&segment.line.b).abs();
                        let hit_back = hit_wall.normal.dot(&direction) > 0.0;
                        let hit_edge = t < 0.01 || t > 0.99;
                        if hit_back || hit_edge {
                            break;
                        }

                        // Go through the portal
                        origin = MapWall::translate(segment.line.b, hit_wall, &new_wall);
                        direction = MapWall::rotate(direction, hit_wall, &new_wall);

                        segments.push(segment.clone());
                        let region = new_wall.region.clone();
                        segment = single_ray_trace(origin.add(&direction), direction, region);
                    }
                }
            }
        }
    }

    segments.push(segment);
    segments
}

pub(crate) fn trace_clear_path_between<'a>(origin: Vector2, target: Vector2, region: &'a MapRegion<'a>) -> Option<Vec<HitResult<'a>>> {
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
pub(crate) fn trace_clear_path_no_portals_between<'a>(origin: Vector2, target: Vector2, region: &'a MapRegion<'a>) -> Option<HitResult<'a>> {
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
pub(crate) fn trace_clear_portal_light(light: &PortalLight, target: Vector2) -> Option<LineSegment2> {
    let ray = LineSegment2::of(light.fake_position, target);

    let ray_hit_portal_pos = ray.intersection(&light.portal_out.line);
    if ray_hit_portal_pos.is_nan() {  // The light does not pass through the portal to the point.
        return None;
    }

    let direction = target.subtract(&ray_hit_portal_pos);

    let ray = LineSegment2::from(ray_hit_portal_pos.add(&direction.tiny()), direction.scale(1.0 - (5.0 * EPSILON)));
    for wall in light.portal_out.region.walls() {
        let hit = wall.line.intersection(&ray);
        if !hit.is_nan() {
            return None;
        }
    }

    Some(ray)
}

/// Sends a ray through a single region until it hits a wall. Without following portals.
pub(crate) fn single_ray_trace<'a>(origin: Vector2, direction: Vector2, region: &'a MapRegion<'a>) -> HitResult<'a> {
    let ray = LineSegment2::from(origin, direction.scale(VIEW_DIST));

    let mut shortest_hit_distance_squared = f64::INFINITY;
    let mut closest_hit_point = Vector2::NAN;
    let mut hit_wall = None;

    for wall in region.walls() {
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
                region,
                line: LineSegment2::of(origin, origin.add(&direction.scale(VIEW_DIST))),
                kind: HitKind::HitNone
            }
        }
        Some(hit_wall) => {
            HitResult {
                region,
                line: LineSegment2::of(origin, closest_hit_point),
                kind: HitKind::HitWall {
                    hit_wall
                }
            }
        }
    };

    hit_result
}


/// How many rays to cast when deciding if a light hits a portal
const PORTAL_SAMPLE_LENGTH: f64 = 1.0 / 5.0;

/// Find the shortest clear path, without following portals, from a point to a wall.
/// Returns None if there is no clear path.
pub(crate) fn find_shortest_path<'a>(region: &'a MapRegion<'a>, pos: Vector2, wall_normal: Vector2, wall: LineSegment2) -> Option<HitResult<'a>> {
    let sample_count = (wall.length() / PORTAL_SAMPLE_LENGTH).floor();
    let mut shortest_path = None;
    let mut shortest_distance = f64::INFINITY;
    for i in 0..(sample_count as i32) {
        let t = i as f64 / sample_count;
        let wall_point = wall.at_t(t);
        let segments = trace_clear_path_between(pos, wall_point, region);
        match segments {
            None => {}
            Some(mut segments) => {
                if segments.len() == 1 {
                    let path = segments.pop().unwrap();
                    let hits_front = path.line.direction().dot(&wall_normal) > EPSILON;
                    if hits_front && path.line.length() < shortest_distance {
                        shortest_distance = path.line.length();
                        shortest_path = Some(path);
                    }
                }

            }
        }
    }

    shortest_path
}


#[derive(Clone)]
pub struct HitResult<'a> {
    pub(crate) region: &'a MapRegion<'a>,
    pub(crate) line: LineSegment2,
    pub(crate) kind: HitKind<'a>
}

impl<'a> HitResult<'a> {
    pub(crate) fn empty(region: &'a MapRegion<'a>, origin: Vector2, direction: Vector2) -> HitResult<'a> {
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
pub(crate) enum HitKind<'a> {
    HitNone,
    HitWall { hit_wall: &'a MapWall<'a> },
    HitPlayer { box_side: LineSegment2 }
}
