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
use std::{default::Default,
          sync::{atomic::{AtomicUsize, Ordering},
                 MutexGuard},
          thread::{self, JoinHandle}};
use types::Object;

/// This is currently set to `0` for testing purposes - bugs get
/// caught much more quickly when the gc runs immediately. A
/// reasonable value would be based off the number of builtin
/// functions, and is probably in the hundreds or low thousands. Emacs
/// uses like 80000 or something, but is also a much larger
/// interpreter with many more builtins.
const INITIAL_GC_THRESHOLD: usize = 0;

lazy_static! {
    pub static ref THE_GC_THREAD: JoinHandle<!> = {
        thread::spawn(gc_thread)
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
    let mut lock = ALLOCED_OBJECTS.lock().unwrap();
    info!("Garbage collecting.");
    let mark = THE_GC_MARK.fetch_add(1, Ordering::Relaxed);
    gc_mark_stack(mark);
    mark_scope(mark);
    sweep(mark, &mut lock);
    update_gc_threshold(&lock);
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
