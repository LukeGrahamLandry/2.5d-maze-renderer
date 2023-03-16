use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::ptr;
use crate::world_data::WorldThing;

pub struct UnsafeLock {
    locked: bool
}

pub static mut SHELF_LOCK: UnsafeLock = UnsafeLock { locked: false };

pub(crate) fn lock_shelves() {
    unsafe {
        SHELF_LOCK.locked = true;
    }
}

pub(crate) fn unlock_shelves() {
    unsafe {
        SHELF_LOCK.locked = false;
    }
}


#[derive(Eq)]
pub struct Shelf<T> {
    cell: RefCell<T>
}

#[derive(Copy)]
pub struct ShelfPtr<T: ?Sized> {
    cell: *const RefCell<T>
}

// only if you call lock
unsafe impl<T> Sync for Shelf<T> {}

pub struct ShelfRef<'b, T: 'b + ?Sized> {
    holder: Option<Ref<'b, T>>,
    ptr: Option<*const T>
}

pub struct ShelfRefMut<'b, T: 'b + ?Sized> {
    holder: Option<RefMut<'b, T>>,
    ptr: Option<*mut T>
}

impl<T: ?Sized> Deref for ShelfRef<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe {
            if SHELF_LOCK.locked {
                &*self.ptr.unwrap()
            } else {
                self.holder.as_ref().unwrap()
            }
        }
    }
}


impl<T: ?Sized> Deref for ShelfRefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe {
            if SHELF_LOCK.locked {
                &*self.ptr.unwrap()
            } else {
                self.holder.as_ref().unwrap()
            }
        }
    }
}

impl<T: ?Sized> DerefMut for ShelfRefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            if SHELF_LOCK.locked {
                &mut *self.ptr.unwrap()
            } else {
                self.holder.as_mut().unwrap()
            }
        }
    }
}


impl<T> Shelf<T> {
    pub(crate) fn new(value: T) -> Shelf<T> where T: Sized {
        Shelf { cell: RefCell::new(value) }
    }

    pub(crate) fn borrow(&self) -> ShelfRef<T> {
        unsafe {
            if SHELF_LOCK.locked {
                ShelfRef {
                    holder: None,
                    ptr: Some(self.peek())
                }
            } else {
                ShelfRef {
                    holder: Some(self.cell.borrow()),
                    ptr: None
                }
            }
        }
    }

    pub(crate) fn borrow_mut(&self) -> ShelfRefMut<T> {
        unsafe {
            if SHELF_LOCK.locked {
                panic!("Cannot borrow_mut while shelves are locked.")
            } else {
                ShelfRefMut {
                    holder: Some(self.cell.borrow_mut()),
                    ptr: None
                }
            }
        }
    }

    pub(crate) fn ptr(&self) -> ShelfPtr<T> {
        ShelfPtr::new(self)
    }

    fn raw_ptr(&self) -> *const RefCell<T> {
        &self.cell as *const RefCell<T>
    }

    pub(crate) fn peek(&self) -> &T {
        unsafe {
            if SHELF_LOCK.locked {
                &*(self.cell.as_ptr() as *const T)
            } else {
                panic!("Cannot peek while shelves are unlocked.")
            }
        }
    }
}

impl<T: ?Sized> ShelfPtr<T> {
    pub(crate) fn new(cell: &Shelf<T>) -> ShelfPtr<T> where T: Sized {
        ShelfPtr { cell: &cell.cell }
    }

    pub(crate) fn null<A>() -> ShelfPtr<A> {
        ShelfPtr { cell: ptr::null() }
    }

    pub(crate) fn borrow(&self) -> ShelfRef<T> {
        unsafe {
            if SHELF_LOCK.locked {
                ShelfRef {
                    holder: None,
                    ptr: Some(self.peek())
                }
            } else {
                ShelfRef {
                    holder: Some(self.get().borrow()),
                    ptr: None
                }
            }
        }
    }

    pub(crate) fn peek(&self) -> &T {
        unsafe {
            if SHELF_LOCK.locked {
                &*(self.get().as_ptr() as *const T)
            } else {
                panic!("Cannot peek while shelves are unlocked.")
            }
        }
    }

    pub(crate) fn borrow_mut(&self) -> ShelfRefMut<T> {
        unsafe {
            if SHELF_LOCK.locked {
                panic!("Cannot borrow_mut while shelves are locked.")
            } else {
                ShelfRefMut {
                    holder: Some(self.get().borrow_mut()),
                    ptr: None
                }
            }
        }
    }

    fn raw_ptr(&self) -> *const RefCell<T>{
        unsafe { &*self.cell as *const RefCell<T> }
    }

    fn get(&self) -> &RefCell<T> {
        unsafe { &*self.cell }
    }

    pub(crate) fn is(&self, other: &Shelf<T>) -> bool where T: Sized {
        other.raw_ptr() == self.raw_ptr()
    }

    pub(crate) fn as_thing(&self) -> Box<ShelfPtr<dyn WorldThing>> where T: 'static + Sized + WorldThing {
        Box::new(ShelfPtr {
            cell: self.cell
        })
    }
}

impl<T: ?Sized> Clone for ShelfPtr<T> {
    fn clone(&self) -> Self {
        ShelfPtr { cell: self.cell }
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

impl<T> Eq for ShelfPtr<T> {}

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

// impl<T> Debug for Shelf<T> where T: Debug {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         self.borrow().fmt(f)
//     }
// }

impl<T> Debug for ShelfPtr<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.borrow().fmt(f)
    }
}
