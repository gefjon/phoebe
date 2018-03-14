use types::{Object, symbol, reference};
use gc::{GcMark, GarbageCollected};
use types::pointer_tagging::{ObjectTag, PointerTag};
use types::conversions::*;
use std::{convert, fmt, ops};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HeapObject {
    gc_marking: GcMark,
    pub val: Object,
}

impl fmt::Display for HeapObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

impl ops::Deref for HeapObject {
    type Target = Object;
    fn deref(&self) -> &Object {
        &self.val
    }
}

impl ops::DerefMut for HeapObject {
    fn deref_mut(&mut self) -> &mut Object {
        &mut self.val
    }
}

impl GarbageCollected for HeapObject {
    fn my_marking(&self) -> &GcMark {
        &self.gc_marking
    }
    fn my_marking_mut(&mut self) -> &mut GcMark {
        &mut self.gc_marking
    }
    fn gc_mark_children(&mut self, mark: GcMark) {
        self.val.gc_mark(mark)
    }
}

impl<'any> convert::From<&'any mut HeapObject> for reference::Reference {
    fn from(h: &mut HeapObject) -> reference::Reference {
        reference::Reference::from(&mut h.val)
    }
}

impl convert::From<*mut HeapObject> for Object {
    fn from(o: *mut HeapObject) -> Object {
        Object(
            ObjectTag::HeapObject.tag(o as u64)
        )
    }
}

impl FromUnchecked<Object> for *mut HeapObject {
    unsafe fn from_unchecked(obj: Object) -> *mut HeapObject {
        debug_assert!(<*mut HeapObject>::is_type(obj));
        <*mut HeapObject>::associated_tag().untag(obj.0) as *mut HeapObject
    }
}

impl FromObject for *mut HeapObject {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::HeapObject
    }
    fn type_name() -> *const symbol::Symbol {
        unimplemented!()
    }
}
