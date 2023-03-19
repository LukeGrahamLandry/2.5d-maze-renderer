use std::hash::{Hash, Hasher};
use std::ops::{Index, IndexMut};
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::new_world::World;
use crate::ray::{Portal, SolidWall};

/// The static arrangement of regions and walls in the world.
/// Once built, it will never mutate which allows reference loops since everything as the same lifetime.
/// Other representations of world data will synchronise their identities by referencing into this structure.
pub(crate) struct Map<'map> {
    regions: Vec<MapRegion<'map>>
}

pub(crate) struct MapRegion<'map> {
    pub(crate) index: usize,
    walls: Vec<MapWall<'map>>,
    lights: Vec<MapLight<'map>>,
    pub(crate) floor_material: Material
}

pub(crate) struct MapWall<'map> {
    pub(crate) index: usize,
    pub(crate) region: &'map MapRegion<'map>,
    pub(crate) line: LineSegment2,
    pub(crate) normal: Vector2,
    pub(crate) next_wall: Option<&'map MapWall<'map>>,
    pub(crate) material: Material,
}

pub(crate) struct MapLight<'map> {
    pub(crate) index: usize,
    pub(crate) region: &'map MapRegion<'map>,
    pub(crate) intensity: Colour,
    pub(crate) pos: Vector2
}

impl<'map: 'walls, 'walls> SolidWall<'map, 'walls> for MapWall<'map> {
    fn portal(&'walls self) -> Option<Portal<'map, 'walls>> {
        match self.next_wall {
            None => { None }
            Some(next_wall) => {
                Some(Portal {
                    from_wall: self,
                    to_wall: next_wall
                })
            }
        }
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn line(&self) -> LineSegment2 {
        self.line
    }

    fn normal(&self) -> Vector2 {
        self.normal
    }

    fn region(&self) -> &'map MapRegion<'map> {
        self.region
    }
}

// If you ever resize any of the vectors, all the references are completely fucked.
// I could switch to Box<[T]> but you can't call into_boxed_slice into one without getting rid of the extra capacity.
// And I don't trust it not to reallocate even if it doesn't have to. It doesn't want uninitialized memory.
// There's some combination of MaybeUninit and raw pointers that would make this work.
// But just having a getter that only gives immutable references and being careful to be safe in this module works too.

impl<'map> Map<'map> {
    pub(crate) fn regions(&'map self) -> &Vec<MapRegion> {
        &self.regions
    }
}

impl<'map> MapRegion<'map> {
    pub(crate) fn walls(&'map self) -> &Vec<MapWall<'map>> {
        &self.walls
    }

    pub(crate) fn lights(&'map self) -> &Vec<MapLight<'map>> {
        &self.lights
    }
}

pub(crate) struct MapBuilder {
    regions: Vec<RegionBuilder>
}

pub(crate) struct RegionBuilder {
    myself_index: usize,
    walls: Vec<WallBuilder>,
    lights: Vec<LightBuilder>,
    floor_material: Material
}

pub(crate) struct WallBuilder {
    myself_index: usize,
    region_index: usize,
    line: LineSegment2,
    normal: Vector2,
    next_region_index: Option<usize>,
    next_wall_index: Option<usize>,
    material: Material,
}

pub(crate) struct LightBuilder {
    region_index: usize,
    myself_index: usize,
    intensity: Colour,
    pos: Vector2
}

impl MapBuilder {
    pub(crate) fn new() -> MapBuilder {
        MapBuilder {
            regions: vec![]
        }
    }

    pub(crate) fn new_region(&mut self, floor_material: Material) -> usize {
        let i = self.regions.len();
        self.regions.push(RegionBuilder {
            myself_index: i,
            walls: vec![],
            lights: vec![],
            floor_material
        });

        i
    }

    pub(crate) fn new_square_region(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, material: Material) -> usize {
        let region = self.new_region(material);

        let walls = LineSegment2::new_square(x1, y1, x2, y2);
        let light_pos = walls[0].a.add(&walls[0].direction().scale(-0.25).add(&walls[2].direction().scale(-0.25)));
        for i in 0..4 {
            self.new_wall(region, walls[i], if i % 2 == 0 { walls[i].normal() } else { walls[i].normal().negate() }, Material::new(0.2, 0.2, 0.9));
        }
        self.new_light(region, Colour::white(), light_pos);

        region
    }

    pub(crate) fn new_wall(&mut self, region_index: usize, line: LineSegment2, normal: Vector2, material: Material) -> usize {
        let mut walls = &mut self.regions[region_index].walls;
        let i = walls.len();
        walls.push(WallBuilder {
            region_index,
            myself_index: i,
            line,
            normal,
            next_region_index: None,
            material,
            next_wall_index: None,
        });
        i
    }

    pub(crate) fn unidirectional_portal(&mut self, from_region: usize, from_wall: usize, to_region: usize, to_wall: usize){
        let mut from_wall = self.regions[from_region].walls.index_mut(from_wall);
        from_wall.next_region_index = Some(to_region);
        from_wall.next_wall_index = Some(to_wall);
    }

    pub(crate) fn bidirectional_portal(&mut self, from_region: usize, from_wall: usize, to_region: usize, to_wall: usize){
        self.unidirectional_portal(from_region, from_wall, to_region, to_wall);
        self.unidirectional_portal(to_region, to_wall, from_region, from_wall);
    }

    pub(crate) fn new_light(&mut self, region_index: usize, intensity: Colour, pos: Vector2){
        let mut lights = &mut self.regions[region_index].lights;

        lights.push(LightBuilder {
            myself_index: lights.len(),
            region_index,
            intensity,
            pos
        });
    }

    pub(crate) fn build<'map>(&self) -> Map<'map> {
        let mut map = Map {
            regions: Vec::new()
        };

        unsafe {
            self.build_into(&mut map);
        }

        map
    }

    unsafe fn build_into(&self, map: *mut Map) {
        // Its very important that these vectors are sized correctly so they never reallocate and references to their elements remain valid.
        (*map).regions = Vec::with_capacity(self.regions.len());
        for (r, region_builder) in self.regions.iter().enumerate() {
            (*map).regions.push(MapRegion {
                index: r,
                walls: Vec::with_capacity(region_builder.walls.len()),
                lights: Vec::with_capacity(region_builder.lights.len()),
                floor_material: region_builder.floor_material
            });
        }

        for (r, region_builder) in self.regions.iter().enumerate() {
            for (w, wall_builder) in region_builder.walls.iter().enumerate() {
                (*map).regions[r].walls.push(MapWall {
                    index: w,
                    region: (*map).regions.index(r),
                    line: wall_builder.line,
                    normal: wall_builder.normal,
                    next_wall: None,
                    material: wall_builder.material,
                })
            }

            for (l, light_builder) in region_builder.lights.iter().enumerate() {
                (*map).regions[r].lights.push(MapLight {
                    index: l,
                    region: (*map).regions.index(r),
                    intensity: light_builder.intensity,
                    pos: light_builder.pos
                })
            }
        }

        for (r, region_builder) in self.regions.iter().enumerate() {
            for (w, wall_builder) in region_builder.walls.iter().enumerate() {
                match (wall_builder.next_wall_index, wall_builder.next_region_index) {
                    (Some(next_wall), Some(next_region)) => {
                        (*map).regions[r].walls.index_mut(w).next_wall = Some((*map).regions[next_region].walls.index(next_wall));
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn from_map(builder: &Map) -> MapBuilder {
        let mut map = MapBuilder {
            regions: vec![],
        };

        map.regions = Vec::with_capacity(builder.regions.len());
        for (r, region_builder) in builder.regions.iter().enumerate() {
            map.regions.push(RegionBuilder {
                myself_index: r,
                walls: Vec::with_capacity(region_builder.walls().len()),
                lights: Vec::with_capacity(region_builder.lights().len()),
                floor_material: region_builder.floor_material
            });
        }

        for (r, region_builder) in builder.regions.iter().enumerate() {
            for (w, wall_builder) in region_builder.walls().iter().enumerate() {
                map.regions[r].walls.push(WallBuilder {
                    myself_index: w,
                    region_index: r,
                    line: wall_builder.line,
                    normal: wall_builder.normal,
                    next_region_index: match wall_builder.next_wall {
                        None => { None }
                        Some(next_wall) => {
                            Some(builder.regions().iter().position(|check| (check as *const MapRegion) == (next_wall.region as *const MapRegion)).unwrap())
                        }
                    },
                    next_wall_index: match wall_builder.next_wall {
                        None => { None }
                        Some(next_wall) => {
                            Some(next_wall.region.walls().iter().position(|check| (check as *const MapWall) == (next_wall as *const MapWall)).unwrap())
                        }
                    },
                    material: wall_builder.material,
                })
            }

            for (l, light_builder) in region_builder.lights().iter().enumerate() {
                map.regions[r].lights.push(LightBuilder {
                    region_index: r,
                    myself_index: l,
                    intensity: light_builder.intensity,
                    pos: light_builder.pos
                })
            }
        }

        map
    }

    // pub(crate) fn from_world(builder: &World) -> MapBuilder{
    //     let mut map = MapBuilder {
    //         regions: vec![],
    //     };
    //
    //     // Its very important that these vectors are sized correctly so they never reallocate and references to their elements remain valid.
    //     map.regions = Vec::with_capacity(builder.regions.len());
    //     for (r, region_builder) in builder.regions.iter().enumerate() {
    //         map.regions.push(RegionBuilder {
    //             myself_index: r,
    //             walls: Vec::with_capacity(region_builder.()),
    //             lights: Vec::with_capacity(region_builder.lights.len()),
    //             floor_material: region_builder.floor_material
    //         });
    //     }
    //
    //     for (r, region_builder) in builder.regions.iter().enumerate() {
    //         let region_builder = region_builder;
    //         for (w, wall_builder) in region_builder.iter_walls().enumerate() {
    //             map.regions[r].walls.push(WallBuilder {
    //                 myself_index: w,
    //                 region_index: r,
    //                 line: wall_builder.line,
    //                 normal: wall_builder.normal,
    //                 next_region_index: match wall_builder.get_next_wall() {
    //                     None => { None }
    //                     Some(next_wall) => {
    //                         let target_r = builder.index_of_region(&next_wall.region).expect("Failed region connection.");
    //                         Some(target_r)
    //                     }
    //                 },
    //                 next_wall_index: match wall_builder.get_next_wall() {
    //                     None => { None }
    //                     Some(next_wall) => {
    //                         let target_r = builder.index_of_region(&next_wall.region).expect("Failed region connection.");
    //                         let target_w = builder.regions[target_r].index_of_wall(&next_wall).expect("Failed wall connection.");
    //                         Some(target_w)
    //                     }
    //                 },
    //                 material: wall_builder.material,
    //             })
    //         }
    //
    //         for (l, light_builder) in region_builder.lights.iter().enumerate() {
    //             map.regions[r].lights.push(LightBuilder {
    //                 region_index: r,
    //                 myself_index: l,
    //                 intensity: light_builder.intensity,
    //                 pos: light_builder.pos
    //             })
    //         }
    //     }
    //
    //     map
    // }
}

#[cfg(test)]
mod tests {
    use crate::world_gen::example_preset;
    use super::*;

    #[test]
    fn immutable_map_builder() {
        // let world = example_preset();
        // let builder_from_world = MapBuilder::from_world(&world);
        // let map = builder_from_world.build();
        // let builder_from_map = MapBuilder::from_map(&map);

        // assert_eq!(world.regions[0].get_wall(0).material, map.regions()[0].walls()[0].material)
    }
}

// TODO: learn how to write macros


impl<'map> Eq for MapRegion<'map> {}
impl<'map> PartialEq for MapRegion<'map> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<'map> Eq for MapWall<'map> {}
impl<'map> PartialEq for MapWall<'map> {
    fn eq(&self, other: &Self) -> bool {
        self.region == other.region && self.index == other.index
    }
}


impl<'map> Eq for MapLight<'map> {}
impl<'map> PartialEq for MapLight<'map> {
    fn eq(&self, other: &Self) -> bool {
        self.region == other.region && self.index == other.index
    }
}

impl<'map> Hash for MapWall<'map> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.index);
        state.write_usize(self.region.index);
    }
}

impl<'map> Hash for MapLight<'map> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.index);
        state.write_usize(self.region.index);
    }
}

unsafe impl<'map> Sync for Map<'map> {}
unsafe impl<'map> Send for Map<'map> {}
