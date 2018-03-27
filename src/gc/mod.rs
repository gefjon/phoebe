use std::sync::atomic;
use std::default::Default;
use stack::gc_mark_stack;
use allocate::{alloced_count, deallocate, Deallocate, ALLOCED_OBJECTS};

static INITIAL_GC_THRESHOLD: usize = 4;

lazy_static! {
    static ref THE_GC_MARK: atomic::AtomicUsize = {
        atomic::AtomicUsize::new(GcMark::default())
    };
    static ref GC_THRESHOLD: atomic::AtomicUsize = {
        atomic::AtomicUsize::new(INITIAL_GC_THRESHOLD)
    };
}

pub type GcMark = usize;

fn should_gc_run() -> bool {
    alloced_count() > GC_THRESHOLD.load(atomic::Ordering::Relaxed)
}

fn update_gc_threshold() {
    let new_thresh = alloced_count() * 2;
    GC_THRESHOLD.store(new_thresh, atomic::Ordering::Relaxed);
}

fn sweep(m: GcMark) {
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

fn mark_scope(m: GcMark) {
    use symbol_lookup::{gc_mark_scope, SYMBOLS_HEAP};
    for &s in SYMBOLS_HEAP.lock().unwrap().values() {
        s.gc_mark(m);
    }
    gc_mark_scope(m);
}

pub fn gc_pass() {
    info!("Garbage collecting.");
    let mark = THE_GC_MARK.fetch_add(1, atomic::Ordering::Relaxed);
    gc_mark_stack(mark);
    mark_scope(mark);
    sweep(mark);
    update_gc_threshold();
}

pub fn gc_maybe_pass() {
    if should_gc_run() {
        gc_pass();
    }
}

pub trait GarbageCollected: Deallocate + ::std::fmt::Display {
    fn my_marking(&self) -> &GcMark;
    fn my_marking_mut(&mut self) -> &mut GcMark;
    fn gc_mark_children(&mut self, mark: GcMark);
    fn gc_mark(&mut self, mark: GcMark) {
        debug!("Marking {}", self);
        if *(self.my_marking()) != mark {
            *(self.my_marking_mut()) = mark;
            self.gc_mark_children(mark);
        }
    }
    fn should_dealloc(&self, current_marking: GcMark) -> bool {
        *(self.my_marking()) != current_marking
    }
}
