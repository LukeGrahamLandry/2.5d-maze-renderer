use std::collections::HashMap;
use crate::material::{Colour, Material};
use crate::mth::{LineSegment2, Vector2};
use crate::player::Player;
use crate::ray::HitResult;
pub use shelf::{Shelf, ShelfPtr};

pub(crate) struct World {
    pub(crate) regions: Vec<Shelf<Region>>,
    pub(crate) player: Shelf<Player>
}

pub(crate) struct Region {
    pub(crate) walls: Vec<Shelf<Wall>>,
    pub(crate) myself: ShelfPtr<Region>,
    pub(crate) lights: Vec<Shelf<ColumnLight>>,
    pub(crate) things: HashMap<u64, ShelfPtr<dyn WorldThing>>,
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
    fn set_region(&self, region: ShelfPtr<Region>);
    fn get_myself(&self) -> ShelfPtr<dyn WorldThing>;
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
    pub(crate) position: Vector2
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

    pub(crate) fn new_wall(&mut self, line: LineSegment2, normal: Vector2, material: Material){
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
        self.add_wall(wall);
    }

    pub(crate) fn new_light(&mut self, intensity: Colour, position: Vector2){
        let light = ColumnLight {
            region: self.myself.clone(),
            myself: ShelfPtr::<Wall>::null(),
            intensity,
            position
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

    pub(crate) fn add_thing(&mut self, thing: ShelfPtr<dyn WorldThing>) {
        let id = thing.borrow().get_id();
        self.things.insert(id, thing);
    }

    pub(crate) fn remove_thing(&mut self, thing: ShelfPtr<dyn WorldThing>) {
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

mod shelf {
    use std::cell::{Ref, RefCell, RefMut};
    use std::hash::{Hash, Hasher};
    use std::ptr;

    pub struct ShelfPtr<T: ?Sized>(*const RefCell<T>);
    pub struct Shelf<T: ?Sized>(RefCell<T>);

    impl<T: ?Sized> Shelf<T> {
        pub(crate) fn new(value: T) -> Shelf<T> where T: Sized {
            Shelf { 0: RefCell::new(value)}
        }

        pub(crate) fn borrow(&self) -> Ref<T> {
            self.0.borrow()
        }

        pub(crate) fn borrow_mut(&self) -> RefMut<T> {
            self.0.borrow_mut()
        }

        pub(crate) fn ptr(&self) -> ShelfPtr<T> {
            ShelfPtr::new(self)
        }

        fn raw_ptr(&self) -> *const RefCell<T> {
            &self.0 as *const RefCell<T>
        }
    }

    impl<T: ?Sized> ShelfPtr<T> {
        pub(crate) fn new(cell: &Shelf<T>) -> ShelfPtr<T> {
            ShelfPtr { 0: &cell.0 }
        }

        pub(crate) fn null<A>() -> ShelfPtr<A> {
            ShelfPtr { 0: ptr::null() }
        }

        pub(crate) fn borrow(&self) -> Ref<T> {
            self.get().borrow()
        }

        pub(crate) fn borrow_mut(&self) -> RefMut<T> {
            self.get().borrow_mut()
        }

        fn raw_ptr(&self) -> *const RefCell<T>{
            unsafe { &*self.0 as *const RefCell<T> }
        }

        fn get(&self) -> &RefCell<T> {
            unsafe { &*self.0 }
        }

        pub(crate) fn is(&self, other: &Shelf<T>) -> bool {
            other.raw_ptr() == self.raw_ptr()
        }
    }

    impl<T> Clone for ShelfPtr<T> {
        fn clone(&self) -> Self {
            ShelfPtr {
                0: self.0.clone()
            }
        }
    }

    impl<T> PartialEq for Shelf<T> {
        fn eq(&self, other: &Self) -> bool {
            self.raw_ptr() == other.raw_ptr()
        }
    }

    impl<T> PartialEq for ShelfPtr<T> {
        fn eq(&self, other: &Self) -> bool {
            self.raw_ptr() == other.raw_ptr()
        }
    }

    impl<T> Hash for Shelf<T> {
        fn hash<H>(&self, hasher: &mut H) where H: Hasher {
            hasher.write_usize(self.raw_ptr() as usize);
        }
    }

    impl<T> Hash for ShelfPtr<T> {
        fn hash<H>(&self, hasher: &mut H) where H: Hasher {
            hasher.write_usize(self.raw_ptr() as usize);
        }
    }
}
