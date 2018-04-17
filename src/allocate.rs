//! These are allocation-related utilities which aren't part of the
//! `GarbageCollected` trait.
//!
//! This module is tiny and probably moving towards being refactored
//! away. Back when `GarbageCollected`, `Allocate` and `Deallocate`
//! were all seperate traits, this module contained the latter two.

use gc::{self, GarbageCollected};
use std::{sync, thread};
use types::{ExpandedObject, Object};

lazy_static! {
    /// A vector of every object which has been allocated on the
    /// heap. The final step of garbage collecting is to iterate
    /// through this vector while filtering out and deallocating any
    /// unused objects.
    pub static ref ALLOCED_OBJECTS: sync::Mutex<Vec<Object>> = { sync::Mutex::new(Vec::new()) };
}

pub fn add_to_alloced(obj: Object) {
    thread::spawn(move || {
        let mut l = ALLOCED_OBJECTS.lock().unwrap();
        l.push(obj);

        if l.len() > gc::GC_THRESHOLD.load(sync::atomic::Ordering::Acquire) {
            gc::THE_GC_THREAD.thread().unpark();
        }
    });
}

#[derive(Fail, Debug)]
/// Represents errors that may occur while deallocating an object.
///
/// This used to have a second variant, `NullPointer`, which was
/// raised when `deallocate` was passed an object which evaluated to a
/// null pointer, but since we use `NonNull` as our pointer type now,
/// that's no longer necessary.
pub enum DeallocError {
    #[fail(display = "Attempt to deallocate a by-value type")]
    ImmediateType,
}

/// This function deallocates an object. It should only be called
/// during garbage collection on an object which appears in
/// `ALLOCED_OBJECTS` and which `should_dealloc`.
pub unsafe fn deallocate(obj: Object) -> Result<(), DeallocError> {
    match ExpandedObject::from(obj) {
        ExpandedObject::Float(_) | ExpandedObject::Immediate(_) | ExpandedObject::Reference(_) => {
            Err(DeallocError::ImmediateType)?
        }
        ExpandedObject::Symbol(s) => GarbageCollected::deallocate(s),
        ExpandedObject::Cons(c) => GarbageCollected::deallocate(c),
        ExpandedObject::Namespace(n) => GarbageCollected::deallocate(n),
        ExpandedObject::HeapObject(h) => GarbageCollected::deallocate(h),
        ExpandedObject::Function(f) => GarbageCollected::deallocate(f),
    }
    Ok(())
}
