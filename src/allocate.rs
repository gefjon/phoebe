use types::{ExpandedObject, Object};
use std::{convert, heap, ptr, sync};
use std::heap::Alloc;

lazy_static! {
    pub static ref ALLOCED_OBJECTS: sync::Mutex<Vec<Object>> = {
        sync::Mutex::new(Vec::new())
    };
}

pub fn alloced_count() -> usize {
    ALLOCED_OBJECTS.lock().unwrap().len()
}

pub trait Allocate<R>
where
    Object: convert::From<*mut Self>,
{
    /// This method should be defined by the receiver but not called
    /// except by `Allocate::allocate`. It is responsible for
    /// converting a source of type `R` into a stack-allocated
    /// `Self`. The method `allocate` calls `alloc_one_and_initialize`
    /// and uses the pointer it created to build an `Object`.
    fn alloc_one_and_initialize(raw: R) -> *mut Self;
    /// This method is the forward-facing export of this
    /// trait. `alloc_one_and_initialize` calls will not be
    /// garbage-collected unless `allocate` is called around them.
    fn allocate(raw: R) -> Object {
        let obj = Object::from(Self::alloc_one_and_initialize(raw));
        ALLOCED_OBJECTS.lock().unwrap().push(obj);
        obj
    }
}

pub trait Deallocate {
    unsafe fn deallocate(obj: *mut Self) -> Result<(), DeallocError>;
}

impl<T, R> Allocate<R> for T
where
    Object: convert::From<*mut T>,
    T: convert::From<R>,
{
    default fn alloc_one_and_initialize(raw: R) -> *mut T {
        let pointer = match heap::Heap.alloc_one() {
            Ok(p) => p.as_ptr(),
            Err(e) => heap::Heap.oom(e),
        };
        unsafe {
            ptr::write(pointer, T::from(raw));
        }
        pointer
    }
}

impl<T> Deallocate for T
where
    Object: convert::From<*mut T>,
{
    default unsafe fn deallocate(obj: *mut T) -> Result<(), DeallocError> {
        match ptr::NonNull::new(obj) {
            Some(p) => {
                ptr::drop_in_place(p.as_ptr());
                heap::Heap.dealloc_one(p);
                Ok(())
            }
            None => Err(DeallocError::NullPointer),
        }
    }
}

#[derive(Fail, Debug)]
pub enum DeallocError {
    #[fail(display = "Attempt to deallocate a null pointer")]
    NullPointer,
    #[fail(display = "Attempt to deallocate a by-value type")]
    ImmediateType,
}

pub unsafe fn deallocate(obj: Object) -> Result<(), DeallocError> {
    match ExpandedObject::from(obj) {
        ExpandedObject::Float(_) | ExpandedObject::Immediate(_) | ExpandedObject::Reference(_) => {
            Err(DeallocError::ImmediateType)
        }
        ExpandedObject::Symbol(s) => Deallocate::deallocate(s.into()),
        ExpandedObject::Cons(c) => Deallocate::deallocate(c),
        ExpandedObject::Namespace(n) => Deallocate::deallocate(n),
        ExpandedObject::HeapObject(h) => Deallocate::deallocate(h),
        ExpandedObject::Function(f) => Deallocate::deallocate(f),
    }
}
