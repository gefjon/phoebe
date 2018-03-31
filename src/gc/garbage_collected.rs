//! Contains the trait `GarbageCollected`, which is implemented by all
//! heap-allocated `Object` variants.

use super::{GcMark, GcRef};
use allocate::add_to_alloced;
use std::{convert, heap::{self, Alloc}, ptr::{self, NonNull}, sync::atomic::Ordering};
use types::Object;

/// All heap-allocated `Object`s implement this trait.
pub trait GarbageCollected
where
    Object: convert::From<GcRef<Self>>,
    Self: Sized,
{
    /// The type calls to `allocate` will be passed. For sized types
    /// this will be `Self`, and for pseudo-unsized types like
    /// `Symbol`, it will be `&[u8]`.
    type ConvertFrom;

    /// This function should be implemented by the reciever. Passed a
    /// `ConvertFrom`, it:
    ///
    /// * allocates a new `Self` on the heap
    ///
    /// * moves the `ConvertFrom` to that new heap-space `Self`
    ///
    /// * returns a `NonNull<Self>`
    fn alloc_one_and_initialize(raw: Self::ConvertFrom) -> NonNull<Self>;

    /// This function is a frontend to `alloc_one_and_initialize`
    /// which handles wrapping the `NonNull` into a `GcRef`.
    fn allocate(raw: Self::ConvertFrom) -> GcRef<Self> {
        let r = Self::alloc_one_and_initialize(raw).into();
        add_to_alloced(Object::from(r));
        r
    }

    unsafe fn deallocate(obj: GcRef<Self>) {
        let nn: NonNull<Self> = obj.into();
        ptr::drop_in_place(nn.as_ptr());
        heap::Heap.dealloc_one(nn);
    }
    fn my_marking(&self) -> &GcMark;

    /// This function is called by `gc_mark` and allows collections to
    /// mark their children. Atoms can write a do-nothing
    /// implementation.
    fn gc_mark_children(&mut self, mark: usize);

    /// Sets `my_marking` to `m` and runs `gc_mark_children`.
    fn gc_mark(obj: &mut GcRef<Self>, m: usize) {
        let old_m = obj.my_marking().swap(m, Ordering::SeqCst);
        if old_m != m {
            obj.gc_mark_children(m);
        }
    }

    /// True iff `my_marking != current_marking`.
    fn should_dealloc(obj: &GcRef<Self>, current_marking: usize) -> bool {
        obj.my_marking().load(Ordering::SeqCst) != current_marking
    }
}
