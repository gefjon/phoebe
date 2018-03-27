use std::{borrow, convert, fmt, ops};
use super::{symbol, Object};
use super::pointer_tagging::{ObjectTag, PointerTag};
use super::conversions::*;

lazy_static! {
    static ref REFERENCE_TYPE_NAME: symbol::SymRef = { ::symbol_lookup::make_symbol(b"reference") };
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Reference(*mut Object);

impl<'any> convert::From<&'any mut Object> for Reference {
    fn from(r: &mut Object) -> Reference {
        Reference(r as *mut Object)
    }
}

impl convert::From<*mut Object> for Reference {
    fn from(r: *mut Object) -> Reference {
        Reference(r)
    }
}

impl convert::From<Reference> for Object {
    fn from(r: Reference) -> Object {
        Object(ObjectTag::Reference.tag(r.0 as u64))
    }
}

impl FromUnchecked<Object> for Reference {
    unsafe fn from_unchecked(obj: Object) -> Reference {
        debug_assert!(Reference::is_type(obj));
        Reference(Reference::associated_tag().untag(obj.0) as *mut Object)
    }
}

impl MaybeFrom<Object> for Reference {
    fn maybe_from(obj: Object) -> Option<Reference> {
        if Reference::is_type(obj) {
            Some(unsafe { Reference::from_unchecked(obj) })
        } else {
            None
        }
    }
}

impl FromObject for Reference {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Reference
    }
    fn type_name() -> symbol::SymRef {
        *REFERENCE_TYPE_NAME
    }
    fn derefs_to(obj: Object) -> bool {
        Self::is_type(obj)
    }
}

impl ops::Deref for Reference {
    type Target = Object;
    fn deref(&self) -> &Object {
        unsafe { &*(self.0) }
    }
}

impl ops::DerefMut for Reference {
    fn deref_mut(&mut self) -> &mut Object {
        unsafe { &mut *(self.0) }
    }
}

impl borrow::Borrow<Object> for Reference {
    fn borrow(&self) -> &Object {
        unsafe { &*(self.0) }
    }
}

impl borrow::BorrowMut<Object> for Reference {
    fn borrow_mut(&mut self) -> &mut Object {
        unsafe { &mut *(self.0) }
    }
}

impl fmt::Debug for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ {:p} -> {} ]", self, self)
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", **self)
    }
}

impl fmt::Pointer for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", self.0)
    }
}
