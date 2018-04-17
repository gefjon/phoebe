use prelude::*;
use std::{convert, fmt, ops};
use types::pointer_tagging::{ObjectTag, PointerTag};

lazy_static! {
    static ref HEAP_OBJECT_TYPE_NAME: GcRef<Symbol> = { symbol_lookup::make_symbol(b"heap-object") };
}

#[derive(Debug)]
pub struct HeapObject {
    gc_marking: GcMark,
    pub val: Object,
}

impl Clone for HeapObject {
    fn clone(&self) -> HeapObject {
        HeapObject::around(self.val)
    }
}

impl HeapObject {
    pub fn around(val: Object) -> HeapObject {
        HeapObject {
            gc_marking: GcMark::default(),
            val,
        }
    }
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
    type ConvertFrom = HeapObject;
    fn alloc_one_and_initialize(h: HeapObject) -> ::std::ptr::NonNull<HeapObject> {
        use std::alloc::{Alloc, Global};
        use std::ptr;
        let nn = Global.alloc_one().unwrap();
        let p = nn.as_ptr();
        unsafe { ptr::write(p, h) };
        nn
    }
    fn my_marking(&self) -> &GcMark {
        &self.gc_marking
    }
    fn gc_mark_children(&mut self, mark: usize) {
        self.val.gc_mark(mark)
    }
}

impl convert::From<GcRef<HeapObject>> for Reference {
    fn from(mut h: GcRef<HeapObject>) -> Reference {
        Reference::from(&mut h.val)
    }
}

impl convert::From<GcRef<HeapObject>> for Object {
    fn from(o: GcRef<HeapObject>) -> Object {
        Object::from_raw(ObjectTag::HeapObject.tag(o.into_ptr() as u64))
    }
}

impl FromUnchecked<Object> for GcRef<HeapObject> {
    unsafe fn from_unchecked(obj: Object) -> Self {
        debug_assert!(Self::is_type(obj));
        GcRef::from_ptr(Self::associated_tag().untag(obj.0) as *mut HeapObject)
    }
}

impl FromObject for GcRef<HeapObject> {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::HeapObject
    }
    fn type_name() -> GcRef<Symbol> {
        *HEAP_OBJECT_TYPE_NAME
    }
}
