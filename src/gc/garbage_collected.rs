use super::{GcMark, GcRef};
use std::heap::{self, Alloc};
use std::ptr::{self, NonNull};
use std::sync::atomic::Ordering;

pub trait GarbageCollected: Sized {
    type ConvertFrom;
    fn alloc_one_and_initialize(raw: Self::ConvertFrom) -> NonNull<Self>;
    fn allocate(raw: Self::ConvertFrom) -> GcRef<Self> {
        Self::alloc_one_and_initialize(raw).into()
    }
    unsafe fn deallocate(obj: GcRef<Self>) {
        let nn: NonNull<Self> = obj.into();
        ptr::drop_in_place(nn.as_ptr());
        heap::Heap.dealloc_one(nn);
    }
    fn my_marking(&self) -> &GcMark;
    fn gc_mark_children(&mut self, mark: usize);
    fn gc_mark(obj: &mut GcRef<Self>, m: usize) {
        let old_m = obj.my_marking().swap(m, Ordering::SeqCst);
        if old_m != m {
            obj.gc_mark_children(m);
        }
    }
    fn should_dealloc(obj: &GcRef<Self>, current_marking: usize) -> bool {
        obj.my_marking().load(Ordering::SeqCst) != current_marking
    }
}
