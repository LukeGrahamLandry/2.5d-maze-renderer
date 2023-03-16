use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::player::Player;
use crate::ray::HitResult;
use crate::shelf::{Shelf, ShelfPtr};

pub(crate) struct World {
    pub(crate) regions: Vec<Shelf<Region>>,
    pub(crate) player: Shelf<Player>
}

pub(crate) struct Region {
    pub(crate) walls: Vec<Shelf<Wall>>,
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
    pub(crate) next_wall: Option<ShelfPtr<Wall>>,
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

impl World {
    pub(crate) fn add_region(&mut self, region: Shelf<Region>){
        self.regions.push(region);
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
}

impl Wall {
    pub(crate) fn bidirectional_portal(a: &mut Wall, b: &mut Wall) {
        a.next_wall = Some(b.myself.clone());
        b.next_wall = Some(a.myself.clone());
    }

    pub(crate) fn add_portal_light(&mut self, light: ShelfPtr<ColumnLight>, relative_location: LineSegment2) {
        self.lights.push(RelativeLight {
            parent: light,
            location: relative_location
        });
    }
}

impl<T> Debug for Shelf<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Shelf")
    }
}

