use self::pointer_tagging::*;
use crate::prelude::*;
use std::{convert, default, fmt, ops};

pub mod cons;
pub mod conversions;
pub mod error;
pub mod function;
pub mod heap_object;
pub mod immediate;
pub mod list;
pub mod namespace;
pub mod number;
mod pointer_tagging;
pub mod reference;
pub mod symbol;

/// Every Phoebe value is represented by an `Object`. `Object`s are
/// NaN-boxed, and the non-`f64` values are pointer-tagged using
/// `ObjectTag`.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Object(u64);

impl Object {
    pub fn quiet_error(e: GcRef<Error>) -> Object {
        Object::from_raw(self::error::ErrorTag::Quiet.tag(e.into_ptr() as u64))
    }
    pub fn loud_error(e: GcRef<Error>) -> Object {
        Object::from_raw(self::error::ErrorTag::Signaling.tag(e.into_ptr() as u64))
    }
    pub fn expand_quiet(self) -> ExpandedObject {
        use std::convert::TryInto;
        match self.try_into() {
            Ok(ex) => ex,
            Err(e) => ExpandedObject::QuietError(e),
        }
    }
    pub fn from_raw(n: u64) -> Object {
        Object(n)
    }
    pub fn into_raw(self) -> u64 {
        self.0
    }
    /// Used by the garbage collector. Returns `true` if this object
    /// should be passed to `allocate::deallocate` - heap objects will
    /// return `true` if their `gc_marking` does not match `mark` and
    /// by-value objects will always return `false`.
    pub fn should_dealloc(self, mark: usize) -> bool {
        match self.expand_quiet() {
            ExpandedObject::Float(_)
            | ExpandedObject::Immediate(_)
            | ExpandedObject::Reference(_) => false,
            ExpandedObject::Cons(c) => c.should_dealloc(mark),
            ExpandedObject::Symbol(s) => s.should_dealloc(mark),
            ExpandedObject::Namespace(n) => n.should_dealloc(mark),
            ExpandedObject::HeapObject(h) => h.should_dealloc(mark),
            ExpandedObject::Function(func) => func.should_dealloc(mark),
            ExpandedObject::QuietError(e) => e.should_dealloc(mark),
        }
    }
    /// Used by the garbage collector - if `self` is a heap object,
    /// this method derefs and marks it so that it will not be
    /// deallocated. For by-value objects, this is a no-op.
    pub fn gc_mark(self, mark: usize) {
        match self.expand_quiet() {
            ExpandedObject::Float(_) | ExpandedObject::Immediate(_) => (),
            ExpandedObject::Reference(r) => (*r).gc_mark(mark),
            ExpandedObject::Cons(c) => c.gc_mark(mark),
            ExpandedObject::Symbol(s) => s.gc_mark(mark),
            ExpandedObject::Namespace(n) => n.gc_mark(mark),
            ExpandedObject::HeapObject(h) => h.gc_mark(mark),
            ExpandedObject::Function(func) => func.gc_mark(mark),
            ExpandedObject::QuietError(e) => e.gc_mark(mark),
        }
    }
    /// This object represents the boolean `false`, or the null-pointer.
    pub fn nil() -> Self {
        Object::from(Immediate::from(false))
    }
    /// This object represents the boolean `true`.
    pub fn t() -> Self {
        Object::from(Immediate::from(true))
    }
    /// A special marker value (of type `Immediate(SpecialMarker)`)
    /// denoting an uninitialized value
    pub fn uninitialized() -> Self {
        Object::from(immediate::SpecialMarker::Uninitialized)
    }
    /// True iff self is exactly `Object::nil()`
    pub fn nilp(self) -> bool {
        self == Object::nil()
    }

    /// True iff self is exactly `Object::uninitialized()`
    pub fn undefinedp(self) -> bool {
        self == Object::uninitialized()
    }

    /// The logical inverse of `undefinedp` - true for any object
    /// other than `Object::uninitialized()`.
    pub fn definedp(self) -> bool {
        !self.undefinedp()
    }

    pub fn eql(self, other: Object) -> bool {
        if let (Some(n), Some(m)) = (
            number::PhoebeNumber::maybe_from(self),
            number::PhoebeNumber::maybe_from(other),
        ) {
            n == m
        } else {
            self == other
        }
    }
    pub fn equal(self, other: Object) -> bool {
        match (self.expand_quiet(), other.expand_quiet()) {
            (ExpandedObject::Reference(r), _) => other.equal(*r),
            (_, ExpandedObject::Reference(r)) => self.equal(*r),
            (ExpandedObject::Cons(a), ExpandedObject::Cons(b)) => *a == *b,
            (ExpandedObject::HeapObject(r), _) => other.equal(**r),
            (_, ExpandedObject::HeapObject(r)) => self.equal(**r),
            _ => self.eql(other),
        }
    }
}

impl ops::Try for Object {
    type Ok = Object;
    type Error = GcRef<Error>;
    fn into_result(self) -> Result<Object, GcRef<Error>> {
        if error::ErrorTag::Signaling.is_of_type(self.into_raw()) {
            Err(unsafe { self.into_unchecked() })
        } else {
            Ok(self)
        }
    }
    fn from_error(e: GcRef<Error>) -> Object {
        Object::loud_error(e)
    }
    fn from_ok(o: Object) -> Object {
        o
    }
}

impl default::Default for Object {
    fn default() -> Object {
        Object::uninitialized()
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expand_quiet())
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.expand_quiet())
    }
}

impl fmt::Display for ExpandedObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ExpandedObject::Float(n) => write!(f, "{}", n),
            ExpandedObject::Reference(r) => write!(f, "{}", r),
            ExpandedObject::Symbol(s) => write!(f, "{}", *s),
            ExpandedObject::Immediate(i) => write!(f, "{}", i),
            ExpandedObject::Cons(c) => write!(f, "{}", c),
            ExpandedObject::Namespace(n) => write!(f, "{}", n),
            ExpandedObject::HeapObject(h) => write!(f, "{}", h),
            ExpandedObject::Function(func) => write!(f, "{}", func),
            ExpandedObject::QuietError(e) => write!(f, "{}", e),
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
            ExpandedObject::Cons(c) => write!(f, "{:?}", *c),
            ExpandedObject::Namespace(n) => write!(f, "{:?}", *n),
            ExpandedObject::HeapObject(h) => write!(f, "{:?}", *h),
            ExpandedObject::Function(func) => write!(f, "{:?}", *func),
            ExpandedObject::QuietError(e) => write!(f, "{:?}", *e),
        }
    }
}

impl convert::From<f64> for Object {
    fn from(f: f64) -> Object {
        Object(f.to_bits())
    }
}

impl convert::TryFrom<Object> for ExpandedObject {
    type Error = GcRef<Error>;
    fn try_from(obj: Object) -> Result<ExpandedObject, GcRef<Error>> {
        if error::ErrorTag::Signaling.is_of_type(obj.into_raw()) {
            return Err(unsafe { obj.into_unchecked() });
        }
        Ok(if f64::is_type(obj) {
            ExpandedObject::Float(unsafe { obj.into_unchecked() })
        } else if <GcRef<Cons>>::is_type(obj) {
            ExpandedObject::Cons(unsafe { obj.into_unchecked() })
        } else if Immediate::is_type(obj) {
            ExpandedObject::Immediate(unsafe { obj.into_unchecked() })
        } else if <GcRef<Symbol>>::is_type(obj) {
            ExpandedObject::Symbol(unsafe { obj.into_unchecked() })
        } else if Reference::is_type(obj) {
            ExpandedObject::Reference(unsafe { obj.into_unchecked() })
        } else if <GcRef<Namespace>>::is_type(obj) {
            ExpandedObject::Namespace(unsafe { obj.into_unchecked() })
        } else if <GcRef<HeapObject>>::is_type(obj) {
            ExpandedObject::HeapObject(unsafe { obj.into_unchecked() })
        } else if <GcRef<Function>>::is_type(obj) {
            ExpandedObject::Function(unsafe { obj.into_unchecked() })
        } else if <GcRef<Error>>::is_type(obj) {
            ExpandedObject::QuietError(unsafe { obj.into_unchecked() })
        } else {
            unreachable!()
        })
    }
}

impl convert::From<Object> for bool {
    fn from(o: Object) -> bool {
        !(o.nilp() || o.undefinedp())
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
    Immediate(Immediate),
    Reference(Reference),
    Symbol(GcRef<Symbol>),
    Cons(GcRef<Cons>),
    Namespace(GcRef<Namespace>),
    HeapObject(GcRef<HeapObject>),
    Function(GcRef<Function>),
    QuietError(GcRef<Error>),
}
