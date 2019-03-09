use super::pointer_tagging::{ObjectTag, PointerTag};
use crate::prelude::*;
use std::{borrow, convert, fmt, ops};

lazy_static! {
    static ref REFERENCE_TYPE_NAME: GcRef<Symbol> = { symbol_lookup::make_symbol(b"reference") };
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Reference(GcRef<Object>);

impl<'any> convert::From<&'any mut Object> for Reference {
    fn from(r: &mut Object) -> Reference {
        Reference(unsafe { GcRef::from_ptr(r as *mut Object) })
    }
}

impl convert::From<*mut Object> for Reference {
    #[cfg_attr(feature = "cargo-clippy", allow(not_unsafe_ptr_arg_deref))]
    fn from(r: *mut Object) -> Reference {
        Reference(unsafe { GcRef::from_ptr(r) })
    }
}

impl convert::From<Reference> for Object {
    fn from(r: Reference) -> Object {
        Object::from_raw(ObjectTag::Reference.tag(r.0.into_ptr() as u64))
    }
}

impl FromUnchecked<Object> for Reference {
    unsafe fn from_unchecked(obj: Object) -> Reference {
        debug_assert!(Reference::is_type(obj));
        Reference::from(Reference::associated_tag().untag(obj.0) as *mut Object)
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
    fn type_name() -> GcRef<Symbol> {
        *REFERENCE_TYPE_NAME
    }
    fn derefs_to(obj: Object) -> bool {
        Self::is_type(obj)
    }
}

impl ops::Deref for Reference {
    type Target = Object;
    fn deref(&self) -> &Object {
        use std::borrow::Borrow;

        self.0.borrow()
    }
}

impl ops::DerefMut for Reference {
    fn deref_mut(&mut self) -> &mut Object {
        use std::borrow::BorrowMut;

        self.0.borrow_mut()
    }
}

impl borrow::Borrow<Object> for Reference {
    fn borrow(&self) -> &Object {
        use std::ops::Deref;

        self.deref()
    }
}

impl borrow::BorrowMut<Object> for Reference {
    fn borrow_mut(&mut self) -> &mut Object {
        use std::ops::DerefMut;

        self.deref_mut()
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
        write!(f, "{:p}", self.0.into_ptr())
    }
}
