//! Phoebe's parallel/concurrent mark-and-sweep garbage collector.
//!
//! TODO: Move away from `usize` as `GcMark` and replace it with
//! `bool`; replace `IS_GC_RUNNING` and `THE_GC_MARK` with a
//! `Mutex<GcInfo>`, where `GcInfo` is a struct that maps
//! `true`/`false` to "white" and "black".

use allocate::deallocate;
use allocate::{alloced_count, ALLOCED_OBJECTS};
use builtins::FINISHED_SOURCING_BUILTINS;
use stack::gc_mark_stack;
use std::{thread, default::Default, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};

/// This is currently set to `0` for testing purposes - bugs get
/// caught much more quickly when the gc runs immediately. A
/// reasonable value would be based off the number of builtin
/// functions, and is probably in the hundreds or low thousands.
const INITIAL_GC_THRESHOLD: usize = 0;

static IS_GC_RUNNING: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref THE_GC_MARK: AtomicUsize = { AtomicUsize::default() };
    /// Whenever we finish evaluating an `Object`, we check to see if
    /// `alloced_count` is larger than `GC_THRESHOLD` and if it is,
    /// spawn a garbage collector thread.
    ///
    /// Future optimization: find some way to base `GC_THRESHOLD` off
    /// of `ALLOCED_OBJECTS`' reserved capacity, to discourage
    /// reallocation.
    static ref GC_THRESHOLD: AtomicUsize = { AtomicUsize::new(INITIAL_GC_THRESHOLD) };
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

/// The garbage collector should run iff sourcing builtins is complete
/// and there are more objects allocated than the current
/// `GC_THRESHOLD` and a garbage collector is not currently running.
fn should_gc_run() -> bool {
    if !(FINISHED_SOURCING_BUILTINS.load(Ordering::Acquire)) {
        debug!("Not finished sourcing builtins; will not gc.");
        return false;
    }
    if IS_GC_RUNNING.load(Ordering::Acquire) {
        debug!("gc already running; will not gc.");
    }
    let gc_thresh = GC_THRESHOLD.load(Ordering::Relaxed);
    let ct = alloced_count();
    if ct < gc_thresh {
        debug!(
            "alloced_count {} < gc_thresh {}; will not gc.",
            ct, gc_thresh
        );
        return false;
    }
    true
}

/// Future optimization: find some way to base `GC_THRESHOLD` off of
/// `ALLOCED_OBJECTS`' reserved capacity, to discourage
/// reallocation.
fn update_gc_threshold() {
    let new_thresh = alloced_count() * 2;
    GC_THRESHOLD.store(new_thresh, Ordering::Relaxed);
}

/// Iterate through all of the allocated objects and filter out any
/// which are not marked "white" (in use).
fn sweep(m: usize) {
    let mut heap = ALLOCED_OBJECTS.lock().unwrap();
    let mut new_heap = Vec::with_capacity(heap.len());
    for obj in heap.drain(..) {
        if obj.should_dealloc(m) {
            debug!("{} is unmarked; deallocating it.", obj);
            unsafe { deallocate(obj).unwrap() }
        } else {
            debug!("{} is marked; keeping it.", obj);
            new_heap.push(obj);
        }
    }
    *heap = new_heap;
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
    if IS_GC_RUNNING.swap(true, Ordering::AcqRel) {
        return;
    }
    info!("Garbage collecting.");
    let mark = THE_GC_MARK.fetch_add(1, Ordering::Relaxed);
    gc_mark_stack(mark);
    mark_scope(mark);
    sweep(mark);
    update_gc_threshold();
    info!("Finished garbage collecting.");
    IS_GC_RUNNING.store(false, Ordering::Release);
}

/// Iff `should_gc_run`, spawn a new garbage collector thread running
/// the function `gc_pass`.
pub fn gc_maybe_pass() {
    debug!("Checking if the gc should run.");
    if should_gc_run() {
        debug!("Spawning a gc thread.");
        thread::spawn(gc_pass);
    } else {
        debug!("Not spawning a gc thread.");
    }
}
