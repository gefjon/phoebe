use std::sync::atomic;
use std::default::Default;
use stack::{current_stack_size, STACK};
use allocate::{ALLOCED_OBJECTS, deallocate, Deallocate};

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
    current_stack_size() > GC_THRESHOLD.load(atomic::Ordering::SeqCst)
}

fn update_gc_threshold() {
    let old_thresh = GC_THRESHOLD.load(atomic::Ordering::SeqCst);
    GC_THRESHOLD.fetch_add(old_thresh, atomic::Ordering::SeqCst);
}

fn mark_stack(m: GcMark) {
    for obj in STACK.lock().unwrap().iter() {
        obj.gc_mark(m)
    }
}

fn sweep(m: GcMark) {
    let mut heap = ALLOCED_OBJECTS.lock().unwrap();
    let mut new_heap = Vec::with_capacity(heap.len());
    for obj in heap.drain(..) {
        if obj.should_dealloc(m) {
            unsafe { deallocate(obj).unwrap() }
        } else {
            new_heap.push(obj);
        }
    }
    *heap = new_heap;
}

fn mark_scope(m: GcMark) {
    use symbol_lookup::{SYMBOLS_HEAP, SCOPE};
    for &s in SYMBOLS_HEAP.lock().unwrap().values() {
        s.gc_mark(m);
    }
    for nmspc in SCOPE.lock().unwrap().iter_mut() {
        nmspc.gc_mark(m);
    }
}

pub fn gc_pass() {
    let mark = THE_GC_MARK.fetch_add(1, atomic::Ordering::SeqCst);
    mark_stack(mark);
    mark_scope(mark);
    sweep(mark);
    update_gc_threshold();
}

pub fn gc_maybe_pass() {
    if should_gc_run() {
        gc_pass();
    }
}

pub trait GarbageCollected: Deallocate {
    fn my_marking(&self) -> &GcMark;
    fn my_marking_mut(&mut self) -> &mut GcMark;
    fn gc_mark_children(&mut self, mark: GcMark);
    fn gc_mark(&mut self, mark: GcMark) {
        if *(self.my_marking()) != mark {
            *(self.my_marking_mut()) = mark;
            self.gc_mark_children(mark);
        }
    }
    fn should_dealloc(&self, current_marking: GcMark) -> bool {
        *(self.my_marking()) != current_marking
    }
}
