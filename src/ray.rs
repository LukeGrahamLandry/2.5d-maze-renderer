use crate::mth;
use crate::mth::{Direction, EPSILON, LineSegment2, Position, Vector2};
use crate::world::{LightKind, LightSource, Region, Wall, World};

impl LightSource {
    pub(crate) fn blocked_by_shadow(&self, region: &Region, hit_pos: &Vector2) -> bool {
        match self.kind {
            LightKind::DIRECT() => {
                region.trace_clear_path_no_portals_between(self.pos, *hit_pos).is_none()
            }
            LightKind::PORTAL { line } => {
                region.trace_clear_portal_light(self, &line, *hit_pos).is_none()
            }
        }

    }
}

const PORTAL_LIMIT: u16 = 15;
pub const VIEW_DIST: f64 = 1000.0;

impl World {
    /// Sends a ray through the world, following portals, and returns a separate line segment for each region it passes through.
    pub(crate) fn ray_trace(&self, start_region: usize, mut origin: Position, mut direction: Direction) -> Vec<RaySegment> {
        let mut segments = vec![];

        let mut region = &self.regions[start_region];
        let mut segment: RaySegment = region.single_ray_trace(origin, direction);
        for _ in 0..PORTAL_LIMIT {
            match segment.hit_wall {
                None => { break; }
                Some(hit_wall) => {
                    let hit_wall = region.get_wall(hit_wall);
                    match hit_wall.portal() {
                        None => { break; }
                        Some(portal)  => {
                            let t = hit_wall.line.t_of(&segment.line.b).abs();
                            let hit_back = hit_wall.normal.dot(&direction) > 0.0;
                            let hit_edge = t < 0.01 || t > 0.99;
                            if hit_back || hit_edge {
                                break;
                            }

                            // Transform through the portal
                            region = &self.regions[portal.to_region];
                            origin = portal.translate(segment.line.b);
                            direction = portal.rotate(direction);

                            segments.push(segment.clone());
                            segment = region.single_ray_trace(origin.add(&direction), direction);
                        }
                    }
                }
            }
        }

        segments.push(segment);
        segments
    }

    pub(crate) fn trace_clear_path_with_portals_between(&self, start_region: usize, origin: Vector2, target: Vector2) -> Option<Vec<RaySegment>> {
        let direction = target.subtract(&origin).normalize();
        let segments = self.ray_trace(start_region, origin, direction);
        let last_hit = segments.last().unwrap();
        let has_clear_path = last_hit.line.b.almost_equal(&target);
        if has_clear_path {
            Some(segments)
        } else {
            None
        }
    }
}

impl Region {
    pub(crate) fn trace_clear_path_no_portals_between(&self, origin: Vector2, target: Vector2) -> Option<RaySegment> {
        let direction = target.subtract(&origin);
        let expected_length_sq = direction.length_sq();
        let direction = direction.normalize();
        let last_hit = self.single_ray_trace(origin, direction);
        let found_length_sq = last_hit.line.direction().length_sq();
        if (found_length_sq - expected_length_sq) > -mth::EPSILON {
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
    pub(crate) fn trace_clear_portal_light(&self, light: &LightSource, portal_line: &LineSegment2, target: Vector2) -> Option<LineSegment2> {
        let ray = LineSegment2::of(light.pos, target);

        let ray_hit_portal_pos = ray.intersection(portal_line);
        if ray_hit_portal_pos.is_nan() {  // The light does not pass through the portal to the point.
            return None;
        }

        let direction = target.subtract(&ray_hit_portal_pos);

        let ray = LineSegment2::from(ray_hit_portal_pos.add(&direction.tiny()), direction.scale(1.0 - (5.0 * EPSILON)));
        for wall in self.walls() {
            let hit = wall.line.intersection(&ray);
            if !hit.is_nan() {
                return None;
            }
        }

        Some(ray)

    }

    /// Sends a ray through a single region until it hits a wall. Without following portals.
    pub(crate) fn single_ray_trace(&self, origin: Vector2, direction: Vector2) -> RaySegment {
        let ray = LineSegment2::from(origin, direction.scale(VIEW_DIST));

        let mut shortest_hit_distance_squared = f64::INFINITY;
        let mut closest_hit_point = Vector2::NAN;
        let mut hit_wall = None;

        for wall in self.walls() {
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
                RaySegment::empty(self, origin, direction)
            }
            Some(hit_wall) => {
                RaySegment::hit(self, hit_wall, LineSegment2::of(origin, closest_hit_point))
            }
        };

        hit_result
    }


    /// How many rays to cast when deciding if a light hits a portal
    const PORTAL_SAMPLE_LENGTH: f64 = 1.0 / 5.0;

    /// Find the shortest clear path, without following portals, from a point to a wall.
    /// Returns None if there is no clear path.
    pub(crate) fn find_shortest_path(&self, pos: Vector2, wall_normal: Vector2, wall: LineSegment2) -> Option<RaySegment> {
        let sample_count = (wall.length() / Region::PORTAL_SAMPLE_LENGTH).floor();
        let mut shortest_path = None;
        let mut shortest_distance = f64::INFINITY;
        for i in 0..(sample_count as i32) {
            let t = i as f64 / sample_count;
            let wall_point = wall.at_t(t);
            let segments = self.trace_clear_path_no_portals_between(pos, wall_point);
            match segments {
                None => {}
                Some(path) => {
                    let hits_front = path.line.direction().dot(&wall_normal) > EPSILON;
                    if hits_front && path.line.length() < shortest_distance {
                        shortest_distance = path.line.length();
                        shortest_path = Some(path);
                    }

                }
            }
        }

        shortest_path
    }
}


#[derive(Clone)]
pub struct RaySegment {
    pub(crate) region: usize,
    pub(crate) line: LineSegment2,
    pub(crate) hit_wall: Option<usize>
}

impl RaySegment {
    pub(crate) fn empty(region: &Region, origin: Vector2, direction: Vector2) -> RaySegment {
        RaySegment {
            region: region.id,
            line: LineSegment2::of(origin, origin.add(&direction)),
            hit_wall: None
        }
    }

    pub(crate) fn hit(region: &Region, wall: &Wall, line: LineSegment2) -> RaySegment {
        assert_eq!(region.id, wall.region);
        RaySegment {
            region: region.id,
            line,
            hit_wall: Some(wall.id)
        }
    }

    fn hit_dist_squared(&self) -> f64 {
        match self.hit_wall {
            None => { f64::INFINITY }
            Some(_) => { self.line.length_sq() }
        }
    }
}

