use std::cell::{Ref, RefCell, RefMut, UnsafeCell};
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

pub struct Shelf<T> {
    cell: Box<UnsafeCell<T>>  // needs to go in a box so the shelf can be in a vector and not break raw pointers into the cell if it resizes
}

#[derive(Copy)]
pub struct ShelfPtr<T: ?Sized> {
    cell: *const UnsafeCell<T>
}

// impl<T> Drop for Shelf<T> {
//     fn drop(&mut self) {
//         println!("drop shelf");
//     }
// }

// only if you call lock
unsafe impl<T> Sync for Shelf<T> {}

pub struct ShelfRef<'b, T: 'b + ?Sized> {
    ptr: &'b UnsafeCell<T>
}

pub struct ShelfRefMut<'b, T: 'b + ?Sized> {
    ptr: &'b UnsafeCell<T>
}

impl<T: ?Sized> Deref for ShelfRef<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe {
            &*self.ptr.get()
        }
    }
}


impl<T: ?Sized> Deref for ShelfRefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe {
            &*self.ptr.get()
        }
    }
}

impl<T: ?Sized> DerefMut for ShelfRefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut* self.ptr.get()
        }
    }
}


impl<T> Shelf<T> {
    pub(crate) fn new(value: T) -> Shelf<T> where T: Sized {
        Shelf { cell: Box::new(UnsafeCell::new(value)) }
    }

    pub(crate) fn borrow(&self) -> ShelfRef<T> {
        unsafe {
            ShelfRef {
                ptr: &self.cell
            }
        }
    }

    pub(crate) fn borrow_mut(&self) -> ShelfRefMut<T> {
        unsafe {
            if SHELF_LOCK.locked {
                panic!("Cannot borrow_mut while shelves are locked.")
            } else {
                ShelfRefMut {
                    ptr: &self.cell
                }
            }
        }
    }

    pub(crate) fn ptr(&self) -> ShelfPtr<T> {
        ShelfPtr::new(self)
    }

    fn raw_ptr(&self) -> *const T {
        self.cell.get()
    }

    pub(crate) fn peek(&self) -> &T {
        unsafe {
            if SHELF_LOCK.locked {
                &*self.cell.get()
            } else {
                panic!("Cannot peek while shelves are unlocked.")
            }
        }
    }
}

impl<T: ?Sized> ShelfPtr<T> {
    pub(crate) fn new(cell: &Shelf<T>) -> ShelfPtr<T> where T: Sized {
        ShelfPtr { cell: cell.cell.as_ref() }
    }

    pub(crate) fn null<A>() -> ShelfPtr<A> {
        ShelfPtr { cell: ptr::null() }
    }

    pub(crate) fn borrow(&self) -> ShelfRef<T> {
        if self.cell.is_null() {
            panic!("cannot borrow null");
        }

        unsafe {
           ShelfRef {
               ptr: &*self.cell
           }
        }
    }

    pub(crate) fn peek(&self) -> &T {
        unsafe {
            if SHELF_LOCK.locked {
                &*self.cell.as_ref().unwrap().get()
            } else {
                panic!("Cannot peek while shelves are unlocked.")
            }
        }
    }

    pub(crate) fn borrow_mut(&self) -> ShelfRefMut<T> {
        if self.cell.is_null() {
            panic!("cannot borrow null");
        }

        unsafe {
            if SHELF_LOCK.locked {
                panic!("Cannot borrow_mut while shelves are locked.")
            } else {
                ShelfRefMut {
                    ptr: &*self.cell
                }
            }
        }
    }

    pub(crate) fn raw_ptr(&self) -> *const T{
        unsafe { &*self.cell.as_ref().unwrap().get() as *const T }
    }

    fn get(&self) -> &UnsafeCell<T> {
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
        if self.cell.is_null() {
            panic!("cannot clone null");
        }
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
