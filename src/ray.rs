use std::marker::PhantomData;
use crate::light_cache::{PortalLight};
use crate::map_builder::{MapRegion, MapWall};
use crate::material::Material;
use crate::mth::{Direction, EPSILON, LineSegment2, Position, Vector2};
use crate::new_world::DynamicRegion;

pub(crate) trait SolidWall<'walls> {
    fn portal(&'walls self) -> Option<Portal<'walls>>;
    fn material(&'walls self) -> &'walls Material;
    fn line(&self) -> LineSegment2;
    fn normal(&self) -> Vector2;
    fn region(&self) -> &MapRegion<'_>;
}

#[derive(Clone, Copy)]
pub(crate) struct Portal<'walls> {
    pub(crate) from_wall: &'walls (dyn SolidWall<'walls> + Sync),
    pub(crate) to_wall: &'walls (dyn SolidWall<'walls> + Sync)
}

impl<'walls> Portal<'walls> {
    pub(crate) fn scale_factor(&self) -> f64 {
        // Calculate ratio of lengths with only one square root makes me feel very clever.
        (self.to_wall.line().length_sq() / self.from_wall.line().length_sq()).sqrt()
    }

    // transform to same position but relative to the new wall, accounting for walls of different sizes.
    pub(crate) fn translate(&self, pos: Position) -> Position {
        let last_offset = pos.subtract(&self.from_wall.line().a);
        let fraction = last_offset.length() / self.from_wall.line().direction().length();
        let new_offset = self.to_wall.line().direction().negate().scale(fraction);

        self.to_wall.line().a.add(&new_offset)
    }

    // TODO: should try scaling the direction as well,
    //       if im not superfluously normalizing it during the ray tracing,
    //       would change the length of the basis unit vector which might look cool
    pub(crate) fn rotate(&self, dir: Direction) -> Direction {
        let rot_offset = self.from_wall.normal().angle_between(&self.to_wall.normal().negate());
        let dir = dir.rotate(rot_offset);
        if dir.dot(&self.to_wall.normal()) > 0.0 {
            dir
        } else {
            dir.negate()
        }
    }
}

const PORTAL_LIMIT: u16 = 15;
pub const VIEW_DIST: f64 = 1000.0;

impl<'map: 'walls, 'walls> DynamicRegion<'map, 'walls> {

}


/// Sends a ray through the world, following portals, and returns a separate line segment for each region it passes through.
pub(crate) fn ray_trace<'map: 'walls, 'walls>(mut origin: Position, mut direction: Direction, region: &'map MapRegion<'map>) -> Vec<RaySegment<'map, 'walls>> {
    let mut segments = vec![];

    let mut segment = single_ray_trace(origin, direction, region);
    for _ in 0..PORTAL_LIMIT {
        match segment.hit_wall {
            None => { break; }
            Some(hit_wall) => {
                match hit_wall.portal() {
                    None => { break; }
                    Some(portal) => {
                        let t = hit_wall.line().t_of(&segment.line.b).abs();
                        let hit_back = hit_wall.normal().dot(&direction) > 0.0;
                        let hit_edge = t < 0.01 || t > 0.99;
                        if hit_back || hit_edge {
                            break;
                        }

                        // Go through the portal
                        origin = portal.translate(segment.line.b);
                        direction = portal.rotate(direction);

                        segments.push(segment.clone());
                        segment = single_ray_trace(origin.add(&direction), direction, portal.to_wall.region());
                    }
                }
            }
        }
    }

    segments.push(segment);
    segments
}

pub(crate) fn trace_clear_path_between<'map: 'walls, 'walls>(origin: Vector2, target: Vector2, region: &'map MapRegion<'map>) -> Option<Vec<RaySegment<'map, 'walls>>> {
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
pub(crate) fn trace_clear_path_no_portals_between<'map: 'walls, 'walls>(origin: Vector2, target: Vector2, region: &'map MapRegion<'map>) -> Option<RaySegment<'map, 'walls>> {
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
pub(crate) fn trace_clear_portal_light<'map: 'walls, 'walls>(light: &'walls PortalLight<'map, 'walls>, target: Vector2) -> Option<LineSegment2> {
    let ray = LineSegment2::of(light.fake_position, target);

    let ray_hit_portal_pos = ray.intersection(&light.portal_out.line());
    if ray_hit_portal_pos.is_nan() {  // The light does not pass through the portal to the point.
        return None;
    }

    let direction = target.subtract(&ray_hit_portal_pos);

    let ray = LineSegment2::from(ray_hit_portal_pos.add(&direction.tiny()), direction.scale(1.0 - (5.0 * EPSILON)));
    for wall in light.portal_out.region().walls() {
        let hit = wall.line.intersection(&ray);
        if !hit.is_nan() {
            return None;
        }
    }

    Some(ray)
}

/// Sends a ray through a single region until it hits a wall. Without following portals.
pub(crate) fn single_ray_trace<'map: 'walls, 'walls>(origin: Vector2, direction: Vector2, region: &'map MapRegion<'map>) -> RaySegment<'map, 'walls> {
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

    let hit_result = match hit_wall {
        None => {
            RaySegment::empty(region, origin, direction)
        }
        Some(hit_wall) => {
            RaySegment::hit(region, hit_wall, LineSegment2::of(origin, closest_hit_point))
        }
    };

    hit_result
}


/// How many rays to cast when deciding if a light hits a portal
const PORTAL_SAMPLE_LENGTH: f64 = 1.0 / 5.0;

/// Find the shortest clear path, without following portals, from a point to a wall.
/// Returns None if there is no clear path.
pub(crate) fn find_shortest_path<'map: 'walls, 'walls>(region: &'map MapRegion<'map>, pos: Vector2, wall_normal: Vector2, wall: LineSegment2) -> Option<RaySegment<'map, 'walls>> {
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
pub struct RaySegment<'map: 'walls, 'walls> {
    pub(crate) region: &'map MapRegion<'map>,
    pub(crate) line: LineSegment2,
    pub(crate) hit_wall: Option<&'walls dyn SolidWall<'walls>>
}

impl<'map: 'walls, 'walls> RaySegment<'map, 'walls> {
    pub(crate) fn empty(region: &'map MapRegion<'map>, origin: Vector2, direction: Vector2) -> RaySegment<'map, 'walls> {
        RaySegment {
            region,
            line: LineSegment2::of(origin, origin.add(&direction)),
            hit_wall: None
        }
    }

    pub(crate) fn hit(region: &'map MapRegion<'map>, wall: &'walls (dyn SolidWall<'walls> + 'walls), line: LineSegment2) -> RaySegment<'map, 'walls> {
        RaySegment {
            region,
            line,
            hit_wall: Some(wall)
        }
    }

    fn hit_dist_squared(&self) -> f64 {
        match self.hit_wall {
            None => { f64::INFINITY }
            Some(_) => { self.line.length_sq() }
        }
    }
}

