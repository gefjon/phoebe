use types::{Object, ExpandedObject};
use types::conversions::*;
use std::{sync, convert, heap, ptr, mem};
// use alloc::allocator::{Alloc, Layout};
use std::heap::Alloc;

lazy_static! {
    pub static ref ALLOCED_OBJECTS: sync::Mutex<Vec<Object>> = {
        sync::Mutex::new(Vec::new())
    };
}

pub trait Allocate<R> {
    fn allocate(raw: R) -> Object;
}

pub trait Deallocate {
    unsafe fn deallocate(obj: *mut Self) -> Result<(), DeallocError>;
}

impl<T, R> Allocate<R> for T
where Object: convert::From<*mut T>,
T: convert::From<R> {
    default fn allocate(raw: R) -> Object {
        let pointer = match heap::Heap.alloc_one() {
            Ok(p) => p.as_ptr(),
            Err(e) => heap::Heap.oom(e),
        };
        unsafe {
            ptr::write(pointer, T::from(raw));
        }
        let obj = Object::from(pointer);
        ALLOCED_OBJECTS.lock().unwrap().push(obj);
        obj
    }
}

impl<T> Deallocate for T
where Object: convert::From<*mut T> {
    default unsafe fn deallocate(obj: *mut T) -> Result<(), DeallocError> {
        match ptr::NonNull::new(obj) {
            Some(p) => {
                ptr::drop_in_place(p.as_ptr());
                heap::Heap.dealloc_one(p);
                Ok(())
            }
            None => Err(DeallocError::NullPointer),
        }
    }
}
    

#[derive(Fail, Debug)]
pub enum DeallocError {
    #[fail(display = "Attempt to deallocate a null pointer")]
    NullPointer,
    #[fail(display = "Attempt to deallocate a by-value type")]
    ImmediateType,
}

pub unsafe fn deallocate(obj: Object) -> Result<(), DeallocError> {
    match ExpandedObject::from(obj) {
        ExpandedObject::Float(_)
            | ExpandedObject::Immediate(_)
            | ExpandedObject::Reference(_) => Err(DeallocError::ImmediateType),
        ExpandedObject::Symbol(s) => Deallocate::deallocate(s),
        ExpandedObject::Cons(c) => Deallocate::deallocate(c),
    }
}
