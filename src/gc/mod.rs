use std::{sync};
use std::default::Default;
use stack::{current_stack_size, STACK};
use allocate::{ALLOCED_OBJECTS, deallocate, Deallocate};

static INITIAL_GC_THRESHOLD: usize = 4;

lazy_static! {
    static ref THE_GC_MARK: sync::Mutex<GcMark> = {
        sync::Mutex::new(GcMark::default())
    };
    static ref GC_THRESHOLD: sync::Mutex<usize> = {
        sync::Mutex::new(INITIAL_GC_THRESHOLD)
    };
}

pub type GcMark = usize;

fn should_gc_run() -> bool {
    current_stack_size() > *(GC_THRESHOLD.lock().unwrap())
}

fn update_gc_threshold() {
    *(GC_THRESHOLD.lock().unwrap()) = current_stack_size() * 2;
}

fn mark_stack(m: GcMark) {
    for obj in STACK.lock().unwrap().iter() {
        obj.gc_mark(m)
    }
}

fn inc_gc_mark(mut m: sync::MutexGuard<GcMark>) {
    *m = m.wrapping_add(1);
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
    unimplemented!()
}

pub fn gc_pass() {
    let mark = THE_GC_MARK.lock().unwrap();
    mark_stack(*mark);
    mark_scope(*mark);
    sweep(*mark);
    inc_gc_mark(mark);
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
