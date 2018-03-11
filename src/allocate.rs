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

#[derive(Fail, Debug)]
#[fail(display = "Allocating an object failed")]
pub struct AllocError {}

fn alllocate<T>(obj: T) -> Result<Object, AllocError>
where Object: convert::From<*mut T> {
    let pointer = heap::Heap.alloc_one().unwrap().as_ptr();
    if pointer.is_null() {
        Err(AllocError {})
    } else {
        unsafe {
            ptr::write(pointer, obj);
        }
        let obj = Object::from(pointer);
        ALLOCED_OBJECTS.lock().unwrap().push(obj);
        Ok(obj)
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
        ExpandedObject::Symbol(s) => dealloc_internal(s),
        ExpandedObject::Cons(c) => dealloc_internal(c),
    }
}

unsafe fn dealloc_internal<T>(to_dealloc: *mut T) -> Result<(), DeallocError>
where Object: convert::From<*mut T> {
    match ptr::NonNull::new(to_dealloc) {
        Some(mut p) => {
            ptr::drop_in_place(p.as_ptr());
            heap::Heap.dealloc_one(p);
            Ok(())
        }
        None => Err(DeallocError::NullPointer),
    }
}
