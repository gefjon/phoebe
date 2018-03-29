use gc::GarbageCollected;
use std::sync;
use types::{ExpandedObject, Object};

lazy_static! {
    pub static ref ALLOCED_OBJECTS: sync::Mutex<Vec<Object>> = { sync::Mutex::new(Vec::new()) };
}

pub fn alloced_count() -> usize {
    ALLOCED_OBJECTS.lock().unwrap().len()
}

#[derive(Fail, Debug)]
pub enum DeallocError {
    #[fail(display = "Attempt to deallocate a by-value type")]
    ImmediateType,
}

pub unsafe fn deallocate(obj: Object) -> Result<(), DeallocError> {
    match ExpandedObject::from(obj) {
        ExpandedObject::Float(_) | ExpandedObject::Immediate(_) | ExpandedObject::Reference(_) => {
            Err(DeallocError::ImmediateType)?
        }
        ExpandedObject::Symbol(s) => GarbageCollected::deallocate(s),
        ExpandedObject::Cons(c) => GarbageCollected::deallocate(c),
        ExpandedObject::Namespace(n) => GarbageCollected::deallocate(n),
        ExpandedObject::HeapObject(h) => GarbageCollected::deallocate(h),
        ExpandedObject::Function(f) => GarbageCollected::deallocate(f),
    }
    Ok(())
}
