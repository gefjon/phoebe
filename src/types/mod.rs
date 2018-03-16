use std::{convert, fmt};
use gc::{GcMark, GarbageCollected};

mod pointer_tagging;
pub mod conversions;
pub mod immediate;
pub mod reference;
pub mod symbol;
pub mod namespace;
pub mod cons;
pub mod heap_object;
pub mod list;
pub mod number;

use self::conversions::*;

/// Every Phoebe value is represented by an `Object`. `Object`s are
/// NaN-boxed, and the non-`f64` values are pointer-tagged using
/// `ObjectTag`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Object(u64);

impl Object {
    /// Used by the garbage collector. Returns `true` if this object
    /// should be passed to `allocate::deallocate` - heap objects will
    /// return `true` if their `gc_marking` does not match `mark` and
    /// by-value objects will always return `false`.
    pub fn should_dealloc(self, mark: GcMark) -> bool {
        match ExpandedObject::from(self) {
            ExpandedObject::Float(_)
                | ExpandedObject::Immediate(_)
                | ExpandedObject::Reference(_) => false,
            ExpandedObject::Cons(c) => unsafe { &*c }.should_dealloc(mark),
            ExpandedObject::Symbol(s) => s.should_dealloc(mark),
            ExpandedObject::Namespace(n) => unsafe { &*n }.should_dealloc(mark),
            ExpandedObject::HeapObject(h) => unsafe { &*h }.should_dealloc(mark),
        }
    }
    /// Used by the garbage collector - if `self` is a heap object,
    /// this method derefs and marks it so that it will not be
    /// deallocated. For by-value objects, this is a no-op.
    pub fn gc_mark(self, mark: GcMark) {
        match ExpandedObject::from(self) {
            ExpandedObject::Float(_) | ExpandedObject::Immediate(_) => (),
            ExpandedObject::Reference(r) => (*r).gc_mark(mark),
            ExpandedObject::Cons(c) => unsafe { &mut *c }.gc_mark(mark),
            ExpandedObject::Symbol(s) => s.gc_mark(mark),
            ExpandedObject::Namespace(n) => unsafe { &mut *n }.gc_mark(mark),
            ExpandedObject::HeapObject(h) => unsafe { &mut *h }.gc_mark(mark),
        }
    }
    /// This object represents the boolean `false`, or the null-pointer.
    pub fn nil() -> Self {
        Object::from(immediate::Immediate::from(false))
    }
    /// This object represents the boolean `true`.
    pub fn t() -> Self {
        Object::from(immediate::Immediate::from(true))
    }
    /// A special marker value (of type `Immediate(SpecialMarker)`)
    /// denoting an uninitialized value
    pub fn uninitialized() -> Self {
        Object::from(
            immediate::SpecialMarker::Uninitialized
        )
    }
    /// True iff self is exactly Object::nil()
    pub fn nilp(self) -> bool {
        self == Object::nil()
    }

    pub fn eql(self, other: Object) -> bool {
        if let (Some(n), Some(m)) = (
            number::PhoebeNumber::maybe_from(self),
            number::PhoebeNumber::maybe_from(other)
        ) {
            n == m
        } else {
            self == other
        }
    }
    pub fn equal(self, other: Object) -> bool {
        match (ExpandedObject::from(self), ExpandedObject::from(other)) {
            (ExpandedObject::Reference(r), _) => other.equal(*r),
            (_, ExpandedObject::Reference(r)) => self.equal(*r),
            (ExpandedObject::Cons(a), ExpandedObject::Cons(b)) => unsafe {
                *a == *b
            }
            (ExpandedObject::HeapObject(r), _) => other.equal(unsafe { **r }),
            (_, ExpandedObject::HeapObject(r)) => self.equal(unsafe { **r }),
            (_, _) => self.eql(other),
        }
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
            ExpandedObject::Symbol(s) => write!(f, "{}", *s),
            ExpandedObject::Immediate(i) => write!(f, "{}", i),
            ExpandedObject::Cons(c) => write!(f, "{}", unsafe { &*c }),
            ExpandedObject::Namespace(n) => write!(f, "{}", unsafe { &*n }),
            ExpandedObject::HeapObject(h) => write!(f, "{}", unsafe { &*h }),
        }
    }
}

impl fmt::Debug for ExpandedObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ExpandedObject::Float(n) => write!(f, "{:?}", n),
            ExpandedObject::Reference(r) => write!(f, "{:?}", r),
            ExpandedObject::Symbol(s) => write!(f, "{:?}", *s),
            ExpandedObject::Immediate(i) => write!(f, "{:?}", i),
            ExpandedObject::Cons(c) => write!(f, "{:?}", unsafe { &*c }),
            ExpandedObject::Namespace(n) => write!(f, "{:?}", unsafe { &*n }),
            ExpandedObject::HeapObject(h) => write!(f, "{:?}", unsafe { &*h }),
        }
    }
}

impl convert::From<f64> for Object {
    fn from(f: f64) -> Object {
        Object(f.to_bits())
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
        } else if <*mut namespace::Namespace>::is_type(obj) {
            ExpandedObject::Namespace(unsafe { obj.into_unchecked() })
        } else if <*mut heap_object::HeapObject>::is_type(obj) {
            ExpandedObject::HeapObject(unsafe { obj.into_unchecked() })
        } else {
            unreachable!()
        }
    }
}

#[derive(Clone, PartialEq)]
/// Many operations on `Object`s start by converting the `Object` into
/// an `ExpandedObject` and then `match`ing over it. This approach
/// allows us to take advantage of Rust's powerful and expressive
/// `match` syntax while still having an `Object` type that fits in a
/// `u64`.
pub enum ExpandedObject {
    Float(f64),
    Immediate(immediate::Immediate),
    Reference(reference::Reference),
    Symbol(symbol::SymRef),
    Cons(*mut cons::Cons),
    Namespace(*mut namespace::Namespace),
    HeapObject(*mut heap_object::HeapObject),
}
