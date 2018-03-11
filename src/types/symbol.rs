use std::{convert};
use super::Object;
use super::pointer_tagging::{ObjectTag, PointerTag};
use super::conversions::*;

pub struct Symbol {
    _unused_data: u64,
}

impl convert::From<*mut Symbol> for Object {
    fn from(s: *mut Symbol) -> Object {
        Object(ObjectTag::Symbol.tag(s as u64))
    }
}

impl FromUnchecked<Object> for *mut Symbol {
    unsafe fn from_unchecked(obj: Object) -> *mut Symbol {
        debug_assert!(<*mut Symbol>::is_type(obj));
        <*mut Symbol>::associated_tag().untag(obj.0) as *mut Symbol
    }
}

impl FromObject for *mut Symbol {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Symbol
    }
    fn type_name() -> *const Symbol {
        unimplemented!()
    }
}
