use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::iter::Map;
use std::slice::Iter;
use std::sync::RwLock;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::ray::HitResult;
use crate::shelf::{Shelf, ShelfPtr, ShelfRef, ShelfRefMut};

pub(crate) struct World {
    pub(crate) regions: Vec<Shelf<Region>>,
    pub(crate) player: Shelf<Player>
}

pub(crate) struct Region {
    walls: Vec<Shelf<Wall>>,
    pub(crate) myself: ShelfPtr<Region>,
    pub(crate) lights: Vec<Shelf<ColumnLight>>,
    pub(crate) things: HashMap<u64, Box<ShelfPtr<dyn WorldThing>>>,
    pub(crate) floor_material: Material
}

pub(crate) struct Wall {
    pub(crate) region: ShelfPtr<Region>,
    pub(crate) myself: ShelfPtr<Wall>,

    pub(crate) line: LineSegment2,
    pub(crate) normal: Vector2,
    next_wall: Option<ShelfPtr<Wall>>,
    pub(crate) material: Material,
    pub(crate) lights: Vec<RelativeLight>
}

pub(crate) trait WorldThing {
    fn collide(&self, origin: Vector2, direction: Vector2) -> HitResult;
    fn get_id(&self) -> u64;
    fn get_region(&self) -> ShelfPtr<Region>;
    fn set_region(&mut self, region: ShelfPtr<Region>);
    fn get_myself(&self) -> Box<ShelfPtr<dyn WorldThing>>;
}

impl dyn WorldThing {
    fn move_to(&mut self, new_region: ShelfPtr<Region>) {
        let old_region = self.get_region();
        old_region.borrow_mut().remove_thing(self.get_myself());
        new_region.borrow_mut().add_thing(self.get_myself());
        self.set_region(new_region);
    }
}

pub(crate) struct ColumnLight {
    pub(crate) region: ShelfPtr<Region>,
    pub(crate) myself: ShelfPtr<ColumnLight>,

    pub(crate) intensity: Colour,
    pub(crate) pos: Vector2
}

pub(crate) struct RelativeLight {
    pub(crate) parent: ShelfPtr<ColumnLight>,
    pub(crate) location: LineSegment2
}

pub(crate) struct Player {
    pub(crate) pos: Vector2,
    pub(crate) look_direction: Vector2,
    pub(crate) move_direction: Vector2,
    pub(crate) region: ShelfPtr<Region>,
    pub(crate) has_flash_light: bool,
    pub(crate) portals: [Option<ShelfPtr<Wall>>; 2],
    pub(crate) bounding_box: [LineSegment2; 4],
    id: u64,
    pub(crate) material: Material,
    pub(crate) needs_render_update: RwLock<bool>,
    pub(crate) myself: ShelfPtr<Player>,
    pub(crate) first_person_rendering: bool
}

impl World {
    pub(crate) fn add_region(&mut self, region: Shelf<Region>){
        self.regions.push(region);
    }

    pub(crate) fn new(regions: Vec<Shelf<Region>>, player_region_index: usize, player_x: f64, player_y: f64) -> World {
        let mut player = Player::new(regions[player_region_index].borrow().myself.clone());
        player.pos.x = player_x;
        player.pos.y = player_y;
        player.update_bounding_box();
        let id = player.id;

        let player = Shelf::new(player);
        player.borrow_mut().myself = player.ptr();
        regions[0].borrow_mut().things.insert(id, player.ptr().as_thing());

        Region::recalculate_lighting(player.borrow().region.clone());

        World {
            player,
            regions
        }
    }
}

impl Region {
    pub(crate) fn new(floor_material: Material) -> Shelf<Region> {
        let region = Region {
            walls: vec![],
            myself: ShelfPtr::<Region>::null(),
            lights: vec![],
            things: Default::default(),
            floor_material
        };

        let region = Shelf::new(region);
        region.borrow_mut().myself = ShelfPtr::new(&region);

        region
    }

    pub(crate) fn new_wall(&mut self, line: LineSegment2, normal: Vector2, material: Material) -> ShelfPtr<Wall> {
        let wall = Wall {
            region: self.myself.clone(),
            myself: ShelfPtr::<Wall>::null(),
            line,
            normal,
            next_wall: None,
            material,
            lights: vec![]
        };

        let wall = Shelf::new(wall);
        wall.borrow_mut().myself = ShelfPtr::new(&wall);
        let ptr = wall.ptr();
        self.add_wall(wall);
        ptr
    }

    pub(crate) fn new_light(&mut self, intensity: Colour, position: Vector2){
        let light = ColumnLight {
            region: self.myself.clone(),
            myself: ShelfPtr::<Wall>::null(),
            intensity,
            pos: position
        };

        let light = Shelf::new(light);
        light.borrow_mut().myself = ShelfPtr::new(&light);
        self.lights.push(light);
    }

    pub(crate) fn add_wall(&mut self, wall: Shelf<Wall>) {
        wall.borrow_mut().region = self.myself.clone();
        self.walls.push(wall);
    }

    pub(crate) fn remove_wall(&mut self, wall: &Wall){
        let mut index = None;
        for (i, check) in self.walls.iter().enumerate() {
            if wall.myself.is(check) {
                index = Some(i);
                break;
            }
        }

        match index {
            None => {}
            Some(index) => { self.walls.remove(index); }
        }
    }

    pub(crate) fn add_thing(&mut self, thing: Box<ShelfPtr<dyn WorldThing>>) {
        let id = thing.borrow().get_id();
        self.things.insert(id, thing);
    }

    pub(crate) fn remove_thing(&mut self, thing: Box<ShelfPtr<dyn WorldThing>>) {
        let id = thing.borrow().get_id();
        self.things.remove(&id);
    }

    pub(crate) fn iter_walls(&self) -> Map<Iter<Shelf<Wall>>, fn(&Shelf<Wall>) -> ShelfRef<Wall>> {
        self.walls.iter().map(|w| {
            w.borrow()
        })
    }

    pub(crate) fn get_wall(&self, i: usize) -> ShelfRef<Wall> {
        self.walls[i].borrow()
    }

    pub(crate) fn mut_wall(&self, i: usize) -> ShelfRefMut<Wall> {
        self.walls[i].borrow_mut()
    }
}



impl Wall {
    pub(crate) fn bidirectional_portal(a: &mut Wall, b: &mut Wall) {
        a.next_wall = Some(b.myself.clone());
        b.next_wall = Some(a.myself.clone());
    }

    pub(crate) fn unidirectional_portal(&mut self, target_portal: &Wall){
        self.next_wall = Some(target_portal.myself.clone());
    }

    pub(crate) fn add_portal_light(&mut self, light: ShelfPtr<ColumnLight>, relative_location: LineSegment2) {
        self.lights.push(RelativeLight {
            parent: light,
            location: relative_location
        });
    }

    pub(crate) fn get_next_wall(&self) -> Option<&ShelfPtr<Wall>>{
        self.next_wall.as_ref()
    }
}

impl Player {
    pub(crate) fn new(start_region: ShelfPtr<Region>) -> Player {
        Player {
            pos: Vector2::zero(),
            look_direction: Vector2::of(0.0, -1.0),
            move_direction: Vector2::zero(),
            region: start_region,
            has_flash_light: false,
            portals: [None, None],
            bounding_box: LineSegment2::new_square(0.0, 0.0, 0.0, 0.0),
            id: 0,
            material: Material::new(1.0, 0.0, 0.0),
            needs_render_update: RwLock::new(true),
            myself: ShelfPtr::<Player>::null(),
            first_person_rendering: false
        }
    }
}

impl WorldThing for Player {
    fn collide(&self, origin: Vector2, direction: Vector2) -> HitResult {
        self.collide_bounding_box(origin, direction)
    }

    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_region(&self) -> ShelfPtr<Region> {
        self.region.clone()
    }

    fn set_region(&mut self, region: ShelfPtr<Region>) {
        self.region = region;
    }

    fn get_myself(&self) -> Box<ShelfPtr<dyn WorldThing>> {
        self.myself.as_thing()
    }
}


impl Debug for dyn WorldThing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "WorldThing")
    }
}
