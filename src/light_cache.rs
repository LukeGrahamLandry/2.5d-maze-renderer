use std::{collections::HashSet, hash::Hash};
use std::collections::HashMap;
use std::hash::Hasher;
use std::ops::Index;

use crate::{mth::{Vector2}, world::World};
use crate::material::{Colour};
use crate::mth::{EPSILON, LineSegment2};
use crate::ray::RaySegment;
use crate::world::{LightKind, LightSource, Portal, Region, Wall};
use crate::world::LightKind::PORTAL;

impl World {
    pub(crate) fn init_lighting(&mut self){
        let portal_hits = self.collect_portal_lights();
        for (id, portal_light) in portal_hits.into_iter() {
            self.add_portal_light(portal_light);
        }

        for region in &mut self.regions {
            region.init_floor_lighting_cache();
        }
    }

    fn collect_portal_lights(&self) -> HashMap<usize, LightSource> {
        let mut portal_hits = HashMap::new();
        for region in 0..self.regions.len() {
            let region = self.get_region(region);
            for light in region.lights(){
                region.trace_portal_light(light, &mut portal_hits);
            }
        }
        portal_hits
    }

    pub(crate) fn update_lighting(&mut self){
        for region in self.regions.iter_mut() {
            let mut portal_lights = vec![];

            for (id, light) in region.lights.iter() {
                match light.kind {
                    LightKind::DIRECT() => {}
                    PORTAL { .. } => {
                        portal_lights.push(*id);
                    }
                }
            }

            for id in portal_lights {
                region.lights.remove(&id);
            }
        }

        self.init_lighting();
    }

    fn add_portal_light(&mut self, portal_light: LightSource) {
        self.regions[portal_light.region].lights.insert(portal_light.id, portal_light);
    }
}

impl Region {
    /// Collect all times that a light hits a portal in its region.
    fn trace_portal_light(&self, light: &LightSource, found: &mut HashMap<usize, LightSource>){
        assert_eq!(self.id, light.region);
        for wall in self.walls() {
            match wall.portal() {
                // If it's not a portal, we ignore it.
                None => {}
                Some(portal) => {
                    // Check a bunch of points on the wall.
                    let sample_count = (wall.line().length() / Region::PORTAL_SAMPLE_LENGTH).floor();
                    for i in 0..(sample_count as i32) {
                        let t = i as f64 / sample_count;
                        let wall_point = wall.line().at_t(t);
                        let dir_to_light = light.pos.subtract(&wall_point);

                        // If it hit the back, it didnt go through the portal
                        let hits_front = dir_to_light.dot(&wall.normal()) > EPSILON;
                        if !hits_front {
                            continue;
                        }

                        if !light.blocked_by_shadow(self, &wall_point) {
                            // if there's a clear path, add it as a portal light in the next region
                            let offset = wall.line().middle().subtract(&light.pos);
                            let offset = portal.rotate(offset);
                            let new_pos = portal.to_wall_line().middle().subtract(&offset);

                            let portal_light = LightSource {
                                id: maze::rand(),
                                region: portal.to_region,
                                intensity: light.intensity,
                                pos: new_pos,
                                kind: PORTAL {
                                    portal_line: portal.to_wall_line()
                                },
                            };
                            found.insert(maze::rand(), portal_light);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn init_floor_lighting_cache(&mut self){
        for x in 0..self.lighting.width {
            for y in 0..self.lighting.height {
                let pos = Vector2::of(x as f64, y as f64).add(&self.lighting.top_left);
                let colour = self.horizontal_surface_colour(pos);
                self.lighting.floor_light_cache[y * self.lighting.width + x] = colour;
            }
        }
    }


    fn check_portal_light(){

    }
}