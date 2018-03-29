use allocate::deallocate;
use allocate::{alloced_count, ALLOCED_OBJECTS};
use builtins::FINISHED_SOURCING_BUILTINS;
use stack::gc_mark_stack;
use std::{thread, default::Default, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};

const INITIAL_GC_THRESHOLD: usize = 4;

static IS_GC_RUNNING: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref THE_GC_MARK: AtomicUsize = { AtomicUsize::default() };
    static ref GC_THRESHOLD: AtomicUsize = { AtomicUsize::new(INITIAL_GC_THRESHOLD) };
}

pub mod garbage_collected;
pub mod gc_ref;

pub use self::garbage_collected::GarbageCollected;
pub use self::gc_ref::GcRef;

pub type GcMark = AtomicUsize;

fn should_gc_run() -> bool {
    FINISHED_SOURCING_BUILTINS.load(Ordering::Acquire)
        && alloced_count() > GC_THRESHOLD.load(Ordering::Relaxed)
}

fn update_gc_threshold() {
    let new_thresh = alloced_count() * 2;
    GC_THRESHOLD.store(new_thresh, Ordering::Relaxed);
}

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
    IS_GC_RUNNING.store(false, Ordering::Release);
}

pub fn gc_maybe_pass() {
    if should_gc_run() {
        thread::spawn(gc_pass);
    }
}
