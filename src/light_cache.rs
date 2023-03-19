use std::{collections::HashSet, hash::Hash};
use std::collections::HashMap;
use std::hash::Hasher;
use std::ops::Index;

use crate::{mth::{Vector2}, world::World};
use crate::material::{Colour};
use crate::mth::LineSegment2;
use crate::ray::RaySegment;
use crate::world::{LightKind, LightSource, Portal, Region, Wall};
use crate::world::LightKind::PORTAL;

impl World {
    pub(crate) fn init_lighting(&mut self){
        let portal_hits = self.collect_portal_lights();
        for (id, portal_light) in portal_hits.into_iter() {
            self.regions[portal_light.region].lights.insert(portal_light.id, portal_light);
        }
    }

    fn collect_portal_lights(&self) -> HashMap<usize, LightSource> {
        let mut portal_hits = HashMap::new();
        for region in 0..self.regions.len() {
            let region = self.get_region(region);
            for light in region.lights(){
                region.trace_direct_light(light, &mut portal_hits);
            }
        }
        portal_hits
    }
}

impl Region {
    fn insert_portal_light(&mut self, light: &LightSource, portal_wall: &Wall, portal: &Portal, path: RaySegment){
        let adjusted_origin = portal.translate(path.line.b);

        let id = maze::rand();
        let portal_light = LightSource {
            id: id,
            region: self.id,
            intensity: light.intensity,
            pos: adjusted_origin,
            kind: PORTAL {
                line: portal_wall.line,
            },
        };
        
        self.lights.insert(id, portal_light);
    }

    /// Collect all times that a direct light in the region hits a portal.
    fn trace_direct_light(&self, light: &LightSource, found: &mut HashMap<usize, LightSource>){
        assert_eq!(self.id, light.region);
        // For every portal, cast a ray from the light to every point on the portal. The first time one hits, we care.
        for wall in self.walls() {
            let line = wall.line;
            let normal = wall.normal;
            match wall.portal() {
                // If it's not a portal, we ignore it.
                None => {}
                Some(portal) => {
                    let segments = self.find_shortest_path(light.pos,normal, line);
                    match segments {
                        // If the light doesn't hit it, we ignore it.
                        None => {}
                        Some(path) => {
                            let portal_light = LightSource {
                                id: maze::rand(),
                                region: portal.to_region,
                                intensity: light.intensity,
                                pos: path.line.get_b(),
                                kind: LightKind::PORTAL {
                                    line: portal.transform.to_line
                                },
                            };
                            found.insert(maze::rand(), portal_light);
                        }
                    }
                }
            }
        }
    }
}