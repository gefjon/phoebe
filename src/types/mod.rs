use std::{convert, fmt};
use gc::{GcMark, GarbageCollected};

mod pointer_tagging;
pub mod conversions;
pub mod immediate;
pub mod reference;
pub mod symbol;
pub mod namespace;
pub mod cons;

use self::conversions::*;

/// Every Phoebe value is represented by an `Object`. `Object`s are
/// NaN-boxed, and the non-`f64` values are pointer-tagged using
/// `ObjectTag`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Object(u64);

impl Object {
    pub fn should_dealloc(self, mark: GcMark) -> bool {
        match ExpandedObject::from(self) {
            ExpandedObject::Float(_)
                | ExpandedObject::Immediate(_)
                | ExpandedObject::Reference(_) => false,
            ExpandedObject::Cons(c) => unsafe { &*c }.should_dealloc(mark),
            ExpandedObject::Symbol(_s) => unimplemented!(),
        }
    }
    pub fn gc_mark(self, mark: GcMark) {
        match ExpandedObject::from(self) {
            ExpandedObject::Float(_) | ExpandedObject::Immediate(_) => (),
            ExpandedObject::Reference(r) => (*r).gc_mark(mark),
            ExpandedObject::Cons(c) => unsafe { &mut *c }.gc_mark(mark),
            ExpandedObject::Symbol(s) => unimplemented!(),
        }
    }
    pub fn nil() -> Self {
        Object::from(immediate::Immediate::from(false))
    }
    pub fn t() -> Self {
        Object::from(immediate::Immediate::from(true))
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", ExpandedObject::from(*self))
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", ExpandedObject::from(*self))
    }
}

impl fmt::Display for ExpandedObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ExpandedObject::Float(n) => write!(f, "{}", n),
            ExpandedObject::Reference(r) => write!(f, "{}", r),
            ExpandedObject::Symbol(s) => unimplemented!(),
            ExpandedObject::Immediate(i) => write!(f, "{}", i),
            ExpandedObject::Cons(c) => write!(f, "{}", unsafe { &*c }),
        }
    }
}

impl fmt::Debug for ExpandedObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ExpandedObject::Float(n) => write!(f, "{:?}", n),
            ExpandedObject::Reference(r) => write!(f, "{:?}", r),
            ExpandedObject::Symbol(s) => unimplemented!(),
            ExpandedObject::Immediate(i) => write!(f, "{:?}", i),
            ExpandedObject::Cons(c) => write!(f, "{:?}", unsafe { &*c }),
        }
    }
}

impl convert::From<Object> for ExpandedObject {
    fn from(obj: Object) -> ExpandedObject {
        if f64::is_type(obj) {
            ExpandedObject::Float(unsafe { obj.into_unchecked() })
        } else if <*mut cons::Cons>::is_type(obj) {
            ExpandedObject::Cons(unsafe { obj.into_unchecked() })
        } else if immediate::Immediate::is_type(obj) {
            ExpandedObject::Immediate(unsafe { obj.into_unchecked() })
        } else if <*mut symbol::Symbol>::is_type(obj) {
            ExpandedObject::Symbol(unsafe { obj.into_unchecked() })
        } else if reference::Reference::is_type(obj) {
            ExpandedObject::Reference(unsafe { obj.into_unchecked() })
        } else {
            unreachable!()
        }
    }
}

#[derive(Clone)]
pub enum ExpandedObject {
    Float(f64),
    Immediate(immediate::Immediate),
    Reference(reference::Reference),
    Symbol(*mut symbol::Symbol),
    Cons(*mut cons::Cons),
}
