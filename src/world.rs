use std::any::Any;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use maze::Pos;
use crate::camera::{ray_direction_for_x, SCREEN_WIDTH};
use crate::ray::{HitKind, HitResult, ray_trace, single_ray_trace, trace_clear_path_between};
use crate::material::{Material, ColumnLight, Colour};

use crate::mth::{EPSILON, LineSegment2, Vector2};
use crate::player::{Player, WorldThing};
use crate::wrappers::{HashLight, HashWall, Shelf, ShelfView};

pub struct World {
    pub(crate) regions: Vec<Shelf<Region>>,
    pub(crate) player: Shelf<Player>
}

impl World {
    pub(crate) fn update(&mut self, delta_time: f64, pressed: &Vec<Keycode>, delta_mouse: i32){
        self.player.borrow_mut().update(&pressed, &self.regions, delta_time, delta_mouse);
    }

    pub(crate) fn on_mouse_click(&mut self, mouse_button: MouseButton) {
        let direction = ray_direction_for_x((SCREEN_WIDTH / 2) as i32, &self.player.borrow().look_direction);
        let hit: HitResult = {
            let segments = ray_trace(self.player.borrow().pos, direction , &self.player.borrow().region);
            segments.last().unwrap().clone()
        };

        match &hit.kind {
            HitKind::None => {}
            HitKind::Player { .. } => {}
            HitKind::Wall {hit_wall, ..} => {
                let new_portal = {
                    let hit_wall = hit_wall.upgrade();
                    let hit_wall = hit_wall.borrow();
                    let half_portal_direction = hit_wall.line.direction().normalize().scale(10.0);
                    let normal = if direction.dot(&hit_wall.normal) < 0.0 {
                        hit_wall.normal
                    } else {
                        hit_wall.normal.negate()
                    };
                    let start_point = hit.line.b.add(&half_portal_direction).add(&normal.scale(10.0));
                    let end_point = hit.line.b.subtract(&half_portal_direction).add(&normal.scale(10.0));

                    let wall = Wall::new(LineSegment2::of(start_point, end_point), normal, &hit_wall.region.upgrade());
                    wall.borrow_mut().material.colour = Colour::new(0.8, 0.3, 0.3);
                    wall
                };  // Drop the borrow of the hit_wall, incase the ray tracing ran out of depth at a portal. Lets us re-borrow in place_portal.


                match mouse_button {
                    MouseButton::Left => {
                        self.place_portal(new_portal, 0, 1);
                    }
                    MouseButton::Right => {
                        self.place_portal(new_portal, 1, 0);
                    }
                    MouseButton::Middle => {
                        self.player.borrow_mut().clear_portal(0);
                        self.player.borrow_mut().clear_portal(1);
                    }
                    _ => { return; }
                }
            }
        }

        let player = self.player.borrow();
        *player.needs_render_update.write().unwrap() = true;
        Region::recalculate_lighting(player.region.clone());
    }

    fn place_portal(&mut self, new_portal: Shelf<Wall>, replacing_index: usize, connecting_index: usize) {
        let mut player = self.player.borrow_mut();

        // If the player already had a portal placed in this slot, remove it.
        player.clear_portal(replacing_index);

        // Put the new portal in the player's slot.
        player.portals[replacing_index] = Some(new_portal.clone());

        // If there's a portal in the other slot, connect them.
        match &player.portals[connecting_index] {
            None => {}
            Some(connecting_portal) => {
                new_portal.borrow_mut().next_wall = Some(connecting_portal.downgrade());
                connecting_portal.borrow_mut().next_wall = Some(new_portal.downgrade());
            }
        }

        // Add the new portal to the world.
        let region = new_portal.borrow().region.upgrade();
        let mut region = region.borrow_mut();
        region.walls.push(new_portal);
    }
}

#[derive(Debug)]
pub(crate) struct Region {
    pub(crate) walls: Vec<Shelf<Wall>>,
    pub(crate) floor_material: Material,
    pub(crate) lights: Vec<Arc<ColumnLight>>,
    pub(crate) things: HashMap<u64, ShelfView<dyn WorldThing>>
}

impl Region {
    pub(crate) fn remove_wall(&mut self, wall: &Shelf<Wall>){
        let mut to_remove = None;
        for (i, w) in self.walls.iter().enumerate() {
            if wall.ptr_eq(w) {
                to_remove = Some(i);
                break;
            }
        }

        match to_remove {
            None => {}
            Some(i) => {
                self.walls.remove(i);
            }
        }
    }

    pub(crate) fn recalculate_lighting(root_region: Shelf<Region>){
        let mut found_lights: HashMap<HashLight, Shelf<Region>> = HashMap::new();
        let mut found_walls: HashSet<HashWall> = HashSet::new();

        Region::find_lights_recursively(root_region.clone(), &mut found_walls, &mut found_lights);

        for (light, region) in found_lights {
            Region::trace_portal_lights(region, &light);
        }
    }

    const PORTAL_SAMPLE_LENGTH: f64 = 1.0 / 5.0;

    pub(crate) fn trace_portal_lights(region: Shelf<Region>, light: &Arc<ColumnLight>) {
        // For every portal, cast a ray from the light to every point on the portal. The first time one hits, we care.
        for wall in &region.borrow().walls {
            let line = wall.borrow().line;
            let normal = wall.borrow().normal;
            let next_wall = wall.borrow().next_wall.clone();
            match next_wall {
                // If it's not a portal, we ignore it.
                None => {}
                Some(next_wall) => {
                    let segments = Region::find_shortest_path(region.clone(), light.pos,normal, line);
                    match segments {
                        // If the light doesn't hit it, we ignore it.
                        None => {}
                        Some(path) => {
                            // Save where the light appears relative to the OUT portal.
                            let next_wall = next_wall.upgrade();
                            {
                                let adjusted_origin = Wall::translate(path.line.b, &*wall.borrow(), &*next_wall.borrow());
                                let adjusted_direction = Wall::rotate(path.line.direction(), &*wall.borrow(), &*next_wall.borrow()).negate();
                                let line = LineSegment2::from(adjusted_origin, adjusted_direction);
                                println!("Save light: {:?} at {:?} on {:?}", light, line, next_wall.borrow());
                                next_wall.borrow_mut().lights.write().unwrap().insert(HashLight::of(light), line);
                            }

                            // TODO: the OUT portal now needs to send the light to all the other portals in its region. With some limit on the recursion.
                        }
                    }
                }
            }
        }
    }

    // TODO: move to ray.rs
    pub(crate) fn find_shortest_path(region: Shelf<Region>, pos: Vector2, wall_normal: Vector2, wall: LineSegment2) -> Option<HitResult> {
        let sample_count = (wall.length() / Region::PORTAL_SAMPLE_LENGTH).floor();
        let mut shortest_path = None;
        let mut shortest_distance = f64::INFINITY;
        for i in 0..(sample_count as i32) {
            let t = i as f64 / sample_count;
            let wall_point = wall.at_t(t);
            let segments = trace_clear_path_between(pos, wall_point, &region);
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

    // this doesn't need a recursion limit, because the HashSet prevents loops.
    // TODO: this will just reset everything in the whole world. Need to be smarter about which can see each other.
    pub(crate) fn find_lights_recursively(region: Shelf<Region>, mut found_walls: &mut HashSet<HashWall>, mut found_lights: &mut HashMap<HashLight, Shelf<Region>>){
        for light in &region.borrow().lights {
            found_lights.insert(HashLight::of(light), region.clone());
        }

        for wall in &region.borrow().walls {
            match &wall.borrow().next_wall {
                None => {}
                Some(next_wall) => {
                    if found_walls.insert(HashWall::of(wall)) {
                        let next_wall = next_wall.upgrade();
                        let next_wall = next_wall.borrow();
                        let next_region = next_wall.region.upgrade();
                        Region::find_lights_recursively(next_region.clone(), &mut found_walls, &mut found_lights);
                    }
                }
            }
        }
    }

    pub(crate) fn new() -> Shelf<Region> {
        Shelf::new(Region {
            walls: vec![],
            floor_material: Material::new(0.0, 0.0, 0.0),
            lights: vec![],
            things: HashMap::with_capacity(1)
        })
    }

    pub(crate) fn new_square(x1: f64, y1: f64, x2: f64, y2: f64) -> Shelf<Region> {
        let region = Region::new();
        {
            let mut m_region = region.borrow_mut();

            let walls = LineSegment2::new_square(x1, y1, x2, y2);
            m_region.walls.push(Wall::new(walls[0], walls[0].normal(), &region));
            m_region.walls.push(Wall::new(walls[1], walls[1].normal().negate(), &region));
            m_region.walls.push(Wall::new(walls[2], walls[2].normal(), &region));
            m_region.walls.push(Wall::new(walls[3], walls[3].normal().negate(), &region));

            // Put a light somewhere random so I can see the shading
            let light = {
                let wall0 = m_region.walls[0].borrow();
                let wall2 = m_region.walls[2].borrow();
                let pos = wall0.line.a.add(&wall0.line.direction().scale(-0.25).add(&wall2.line.direction().scale(-0.25)));
                ColumnLight {
                    pos,
                    intensity: Colour::white()
                }
            };
            m_region.lights.push(Arc::new(light));
        }

        region
    }
}

#[derive(Debug)]
pub(crate) struct Wall {
    pub(crate) line: LineSegment2,
    pub(crate) normal: Vector2,
    pub(crate) region: ShelfView<Region>,
    pub(crate) next_wall: Option<ShelfView<Wall>>,
    pub(crate) material: Material,
    pub(crate) lights: RwLock<HashMap<HashLight, LineSegment2>>  // lights that are on the other side of the portal -> relative position behind the portal
}

impl Wall {
    pub(crate) fn new(line: LineSegment2, normal: Vector2, region: &Shelf<Region>) -> Shelf<Wall> {
        let wall = Wall {
            region: region.downgrade(),
            next_wall: None,
            normal,
            line,
            material: Material::new(0.2, 0.8, 0.2),
            lights: RwLock::new(HashMap::new())
        };
        Shelf::new(wall)
    }

    pub(crate) fn is_portal(&self) -> bool {
        self.next_wall.is_some()
    }

    pub(crate) fn scale_factor(from: &Wall, to: &Wall) -> f64 {
        to.line.length() / from.line.length()
    }

    // transform to same position but relative to the new wall, accounting for walls of different sizes.
    pub(crate) fn translate(pos: Vector2, from: &Wall, to: &Wall) -> Vector2 {
        let last_offset = pos.subtract(&from.line.a);
        let fraction = last_offset.length() / from.line.direction().length();
        let new_offset = to.line.direction().negate().scale(fraction);

        to.line.a.add(&new_offset)
    }

    pub(crate) fn rotate(direction: Vector2, from: &Wall, to: &Wall) -> Vector2 {
        let rot_offset = from.normal.angle_between(&to.normal.negate());
        let dir = direction.rotate(rot_offset);
        if dir.dot(&to.normal) > 0.0 {
            dir
        } else {
            dir.negate()
        }
    }
}

