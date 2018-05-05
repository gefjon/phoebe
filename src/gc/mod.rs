//! Phoebe's parallel/concurrent mark-and-sweep garbage collector.
//!
//! TODO: Move away from `usize` as `GcMark` and replace it with
//! `bool`; replace `IS_GC_RUNNING` and `THE_GC_MARK` with a
//! `Mutex<GcInfo>`, where `GcInfo` is a struct that maps
//! `true`/`false` to "white" and "black".

use allocate::deallocate;
use allocate::ALLOCED_OBJECTS;
use builtins::make_builtins_once;
use stack::gc_mark_stack;
use std::{
    default::Default, sync::{
        atomic::{AtomicUsize, Ordering}, MutexGuard,
    },
    thread::{self, JoinHandle},
};
use types::Object;

#[cfg(test)]
use std::sync;

/// The garbage collector's stack size, in bytes. This doesn't need to
/// be particularly large; the 2MiB default is excessive.
const GARBAGE_COLLECTOR_STACK_SIZE: usize = 32 * 1024;

/// This is currently set to `0` for testing purposes - bugs get
/// caught much more quickly when the gc runs immediately. A
/// reasonable value would be based off the number of builtin
/// functions, and is probably in the hundreds or low thousands. Emacs
/// uses like 80000 or something, but is also a much larger
/// interpreter with many more builtins.
const INITIAL_GC_THRESHOLD: usize = 0;

#[cfg(test)]
lazy_static! {
    /// `GC_SIGNAL_TUPLE.0` is a `Mutex<bool>` representing the
    /// garbage collector having run, and `.1` is a `Condvar` which
    /// signals whenever the garbage collector runs.
    pub static ref GC_SIGNAL_TUPLE: (sync::Mutex<bool>, sync::Condvar) = {
        (sync::Mutex::new(false), sync::Condvar::new())
    };
}

lazy_static! {
    pub static ref THE_GC_THREAD: JoinHandle<!> = {
        thread::Builder::new()
            .name("Garbage collector".to_owned())
            .stack_size(GARBAGE_COLLECTOR_STACK_SIZE)
            .spawn(gc_thread)
            .unwrap()
    };
    static ref THE_GC_MARK: AtomicUsize = { AtomicUsize::default() };
    /// Whenever we finish evaluating an `Object`, we check to see if
    /// `alloced_count` is larger than `GC_THRESHOLD` and if it is,
    /// spawn a garbage collector thread.
    ///
    /// Future optimization: find some way to base `GC_THRESHOLD` off
    /// of `ALLOCED_OBJECTS`' reserved capacity, to discourage
    /// reallocation.
    pub static ref GC_THRESHOLD: AtomicUsize = { AtomicUsize::new(INITIAL_GC_THRESHOLD) };
}

pub mod garbage_collected;
pub mod gc_ref;

pub use self::garbage_collected::GarbageCollected;
pub use self::gc_ref::GcRef;

/// This could easily be changed to `AtomicBool` - there are only two
/// states, which in gc theory are called "white" and "black". A
/// `bool` feels unintuitive because the two swap after each garbage
/// collection, meaning that half of the time the mark `true` would
/// mean "white" (in use, keep), but the other half of the time it
/// would mean "black" (not in use, deallocate).
pub type GcMark = AtomicUsize;

/// Future optimization: find some way to base `GC_THRESHOLD` off of
/// `ALLOCED_OBJECTS`' reserved capacity, to discourage
/// reallocation.
fn update_gc_threshold(alloced: &MutexGuard<Vec<Object>>) {
    let new_thresh = alloced.len() * 2;
    GC_THRESHOLD.store(new_thresh, Ordering::Relaxed);
}

/// Iterate through all of the allocated objects and filter out any
/// which are not marked "white" (in use).
fn sweep(m: usize, heap: &mut MutexGuard<Vec<Object>>) {
    let mut n_removed: usize = 0;
    let mut new_heap = Vec::with_capacity(heap.len());
    for obj in (*heap).drain(..) {
        if obj.should_dealloc(m) {
            debug!("{} is unmarked; deallocating it.", obj);
            unsafe { deallocate(obj).unwrap() };
            n_removed += 1;
        } else {
            debug!("{} is marked; keeping it.", obj);
            new_heap.push(obj);
        }
    }
    **heap = new_heap;
    info!("Finished sweeping; deallocated {} objects.", n_removed);
}

fn mark_scope(m: usize) {
    use symbol_lookup::{gc_mark_scope, SYMBOLS_HEAP};
    for &s in SYMBOLS_HEAP.lock().unwrap().values() {
        s.gc_mark(m);
    }
    gc_mark_scope(m);
}

/// This is the function which gc threads run with. It will exit
/// immediately if another garbage collector is already running;
/// otherwise it will mark all accessible objects and deallocate any
/// others.
pub fn gc_pass() {
    info!("Garbage collecting.");

    {
        let mut lock = ALLOCED_OBJECTS.lock().unwrap();
        debug!("Acquired the ALLOCED_OBJECTS lock");
        let mark = THE_GC_MARK.fetch_add(1, Ordering::Relaxed);
        gc_mark_stack(mark);
        mark_scope(mark);
        sweep(mark, &mut lock);
        update_gc_threshold(&lock);
        debug!("Dropping the ALLOCED_OBJECTS lock");
    }

    #[cfg(test)]
    {
        let (ref mutex, ref cond_var) = *GC_SIGNAL_TUPLE;

        *(mutex.lock().unwrap()) = true;
        cond_var.notify_all();
    }

    info!("Finished garbage collecting.");
}

fn gc_thread() -> ! {
    make_builtins_once();
    loop {
        {
            gc_pass();
        }
        thread::park();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use allocate::{ALLOCATOR_SIGNAL_TUPLE, ALLOCED_OBJECTS};
    use prelude::*;
    use stack;
    #[test]
    fn something_gets_deallocated() {
        let dead_beef = stack::with_stack(|s| {
            let dead_beef: Object =
                HeapObject::allocate(HeapObject::around(Object::from(0xdead_beefusize))).into();
            s.push(dead_beef);

            dead_beef
        });

        {
            let (ref al_mutex, ref al_cond_var) = *ALLOCATOR_SIGNAL_TUPLE;
            let mut lock = al_mutex.lock().unwrap();

            while *lock != dead_beef {
                lock = al_cond_var.wait(lock).unwrap();
            }
        }

        {
            let a_o = ALLOCED_OBJECTS.lock().unwrap();
            assert!(a_o.contains(&dead_beef));
        }

        assert_eq!(stack::pop().unwrap(), dead_beef);

        {
            let (ref gc_mutex, ref gc_cond_var) = *GC_SIGNAL_TUPLE;
            let mut lock = gc_mutex.lock().unwrap();

            // We wait through two gc cycles in case one has already
            // started - the already-in-progress one may not
            // deallocate `dead_beef`, but the next one must.
            for _ in 0..2 {
                *lock = false;

                THE_GC_THREAD.thread().unpark();

                while !*lock {
                    lock = gc_cond_var.wait(lock).unwrap();
                }
            }
        }
        {
            let a_o = ALLOCED_OBJECTS.lock().unwrap();
            assert!(!(a_o.contains(&dead_beef)));
        }
    }
}
