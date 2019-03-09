//! These are allocation-related utilities which aren't part of the
//! `GarbageCollected` trait.
//!
//! This module is tiny and probably moving towards being refactored
//! away. Back when `GarbageCollected`, `Allocate` and `Deallocate`
//! were all seperate traits, this module contained the latter two.

use crate::gc::{self, GarbageCollected};
use crate::types::{ExpandedObject, Object};
use std::{
    sync::{self, atomic, mpsc, Mutex},
    thread,
};

/// The allocator's stack size, in bytes. This doesn't need to be
/// particularly large; the 2MiB default is excessive.
const ALLOCATOR_THREAD_STACK_SIZE: usize = 16 * 1024;

#[cfg(test)]
lazy_static! {
    pub static ref ALLOCATOR_SIGNAL_TUPLE: (Mutex<Object>, sync::Condvar) = {
        (
            sync::Mutex::new(Object::uninitialized()),
            sync::Condvar::new(),
        )
    };
}

lazy_static! {
    /// A vector of every object which has been allocated on the
    /// heap. The final step of garbage collecting is to iterate
    /// through this vector while filtering out and deallocating any
    /// unused objects.
    pub static ref ALLOCED_OBJECTS: sync::Mutex<Vec<Object>> = { sync::Mutex::new(Vec::new()) };

    /// The garbage collector runs in a seperate thread and must
    /// maintain a lock on `ALLOCED_OBJECTS` while it is running, but
    /// we don't want any thread which allocates anything to
    /// block. The solution is a special allocator thread
    static ref JUST_ALLOCATED: Mutex<mpsc::Sender<Object>> = {
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .name("Allocator".to_owned())
            .stack_size(ALLOCATOR_THREAD_STACK_SIZE)
            .spawn(move || {
                for o in receiver.iter() {
                    let ct = {
                        let mut alloced = ALLOCED_OBJECTS.lock().unwrap();
                        alloced.push(o);
                        alloced.len()
                    };

                    #[cfg(test)]
                    {
                        let (ref mutex, ref cond_var) = *ALLOCATOR_SIGNAL_TUPLE;
                        *(mutex.lock().unwrap()) = o;
                        cond_var.notify_all();
                    }

                    if ct > gc::GC_THRESHOLD.load(atomic::Ordering::Relaxed) {
                        gc::THE_GC_THREAD.thread().unpark();
                    }
                }
            })
            .unwrap();
        Mutex::new(sender)
    };
}

thread_local! {
    static JUST_ALLOCATED_SENDER: mpsc::Sender<Object> = {
        JUST_ALLOCATED.lock().unwrap().clone()
    };
}

/// Every time we allocate an `Object` with heap data, we call
/// `add_to_alloced` on the new `Object`. That puts it into the
/// `ALLOCED_OBJECTS` so that the garbage collector can find it.
pub fn add_to_alloced(obj: Object) {
    JUST_ALLOCATED_SENDER.with(|s| s.send(obj).unwrap());
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
    match obj.expand_quiet() {
        ExpandedObject::Float(_) | ExpandedObject::Immediate(_) | ExpandedObject::Reference(_) => {
            Err(DeallocError::ImmediateType)?
        }
        ExpandedObject::Symbol(s) => GarbageCollected::deallocate(s),
        ExpandedObject::Cons(c) => GarbageCollected::deallocate(c),
        ExpandedObject::Namespace(n) => GarbageCollected::deallocate(n),
        ExpandedObject::HeapObject(h) => GarbageCollected::deallocate(h),
        ExpandedObject::Function(f) => GarbageCollected::deallocate(f),
        ExpandedObject::QuietError(e) => GarbageCollected::deallocate(e),
    }
    Ok(())
}
