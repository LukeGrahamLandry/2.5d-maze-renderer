use std::cell::{Ref, RefCell, RefMut};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};
use crate::material::ColumnLight;
use crate::player::WorldThing;
use crate::world::Wall;

#[derive(Debug)]
pub(crate) struct HashLight(Arc<ColumnLight>);

impl HashLight {
    pub(crate) fn of(x: &Arc<ColumnLight>) -> HashLight {
        HashLight {0: x.clone() }
    }
}

impl PartialEq for HashLight {
    fn eq(&self, other: &HashLight) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for HashLight {}

impl Hash for HashLight {
    fn hash<H>(&self, hasher: &mut H) where H: Hasher {
        hasher.write_usize(Arc::as_ptr(&self.0) as usize);
    }
}

impl Deref for HashLight {
    type Target = Arc<ColumnLight>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct HashWall(Shelf<Wall>);

impl HashWall {
    pub(crate) fn of(x: &Shelf<Wall> ) -> HashWall {
        HashWall {0: x.clone() }
    }
}

impl PartialEq for HashWall {
    fn eq(&self, other: &HashWall) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

impl Eq for HashWall {}

impl Hash for HashWall {
    fn hash<H>(&self, hasher: &mut H) where H: Hasher {
        hasher.write_usize(self.0.hash_ptr());
    }
}

impl Deref for HashWall {
    type Target = Shelf<Wall>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub(crate) struct Shelf<T: ?Sized + 'static> {
    data: Arc<RwLock<T>>,
}

impl<T: ?Sized> Shelf<T> {
    pub(crate) fn new(value: T) -> Shelf<T> where T: Sized {
        Shelf {
            data: Arc::new(RwLock::new(value))
        }
    }

    pub(crate) fn borrow(&self) -> RwLockReadGuard<'_, T> {
        self.data.read().unwrap()
    }

    pub(crate) fn borrow_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.data.write().unwrap()
    }

    pub(crate) fn downgrade(&self) -> ShelfView<T> {
        ShelfView {
            data: Arc::downgrade(&self.data)
        }
    }

    pub(crate) fn ptr_eq(&self, other: &Shelf<T>) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    pub(crate) fn hash_ptr(&self) -> usize {
        Arc::as_ptr(&self.data) as *const () as usize
    }
}

impl<T: ?Sized> Clone for Shelf<T> {
    fn clone(&self) -> Self {
        Shelf { data: self.data.clone() }
    }
}

#[derive(Debug)]
pub(crate) struct ShelfView<T: ?Sized> {
    data: Weak<RwLock<T>>
}

impl<T: ?Sized> ShelfView<T> {
    pub(crate) fn upgrade(&self) -> Shelf<T> {
        Shelf { data: self.data.upgrade().unwrap() }
    }

    // todo: look at CoerceUnsized
    pub(crate) fn to_thing(&self) -> ShelfView<dyn WorldThing> where T: 'static + Sized + WorldThing {
        let data = self.clone().data as Weak<RwLock<dyn WorldThing>>;
        ShelfView { data }
    }
}

impl<T> Clone for ShelfView<T> {
    fn clone(&self) -> Self {
        ShelfView { data: self.data.clone() }
    }
}

unsafe impl<T: ?Sized> Sync for Shelf<T> {}
unsafe impl<T: ?Sized> Sync for ShelfView<T> {}



