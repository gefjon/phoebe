use crate::prelude::*;
use crate::types::pointer_tagging::{ObjectTag, PointerTag};
use std::{convert, fmt};

const IMMEDIATE_TAG_MASK: u64 = 0xffff << 32;

lazy_static! {
    static ref IMMEDIATE_TYPE_NAME: GcRef<Symbol> = { symbol_lookup::make_symbol(b"immediate") };
    static ref SPECIAL_MARKER_TYPE_NAME: GcRef<Symbol> =
        { symbol_lookup::make_symbol(b"special-marker") };
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Immediate {
    Bool(bool),
    Integer(i32),
    UnsignedInt(usize),
    SpecialMarker(SpecialMarker),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
/// This enum represents special values an `Object` can hold. It is
/// [repr(u32)] because all `Immediate` values must fit in a `u32`;
/// the other bytes are used as a tag.
pub enum SpecialMarker {
    /// The default `Object`; using it as a value represents an error
    /// condition.
    Uninitialized,
}

impl fmt::Display for SpecialMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpecialMarker::Uninitialized => write!(f, "UNINITIALIZED"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u64)]
pub enum ImmediateTag {
    Bool,
    Integer,
    UnsignedInt,
    SpecialMarker,
}

impl FromUnchecked<Object> for SpecialMarker {
    unsafe fn from_unchecked(obj: Object) -> SpecialMarker {
        debug_assert!(SpecialMarker::is_type(obj));
        match ImmediateTag::SpecialMarker.untag(obj.0) as u32 {
            n if (SpecialMarker::Uninitialized as u32) == n => SpecialMarker::Uninitialized,
            n => panic!("{} is not a valid SpecialMarker", n),
        }
    }
}

impl FromObject for SpecialMarker {
    type Tag = ImmediateTag;
    fn associated_tag() -> ImmediateTag {
        ImmediateTag::SpecialMarker
    }
    fn type_name() -> GcRef<Symbol> {
        *SPECIAL_MARKER_TYPE_NAME
    }
}

impl convert::From<ImmediateTag> for u64 {
    fn from(t: ImmediateTag) -> u64 {
        (t as u64) << 32
    }
}

impl PointerTag for ImmediateTag {
    fn mask_bits() -> u64 {
        IMMEDIATE_TAG_MASK
    }
    fn parent_mask() -> u64 {
        ObjectTag::parent_mask() ^ ObjectTag::mask_bits()
    }
    fn parent_tag() -> u64 {
        ObjectTag::Immediate.tag(0)
    }
}

impl FromUnchecked<Object> for Immediate {
    unsafe fn from_unchecked(obj: Object) -> Immediate {
        debug_assert!(Immediate::is_type(obj));
        if i32::is_type(obj) {
            Immediate::Integer(i32::from_unchecked(obj))
        } else if bool::is_type(obj) {
            Immediate::Bool(bool::from_unchecked(obj))
        } else if usize::is_type(obj) {
            Immediate::UnsignedInt(usize::from_unchecked(obj))
        } else if SpecialMarker::is_type(obj) {
            Immediate::SpecialMarker(SpecialMarker::from_unchecked(obj))
        } else {
            panic!("Immediate::from_unchecked on a non-Immediate value")
        }
    }
}

impl FromObject for Immediate {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Immediate
    }
    fn type_name() -> GcRef<Symbol> {
        *IMMEDIATE_TYPE_NAME
    }
}

impl convert::From<Immediate> for Object {
    fn from(i: Immediate) -> Object {
        Object::from_raw(match i {
            Immediate::Bool(b) => ImmediateTag::Bool.tag(b as u64),
            Immediate::Integer(n) => ImmediateTag::Integer.tag(u64::from(n as u32)),
            Immediate::UnsignedInt(n) => ImmediateTag::UnsignedInt.tag(n as u64),
            Immediate::SpecialMarker(s) => ImmediateTag::SpecialMarker.tag(u64::from(s as u32)),
        })
    }
}

impl convert::From<bool> for Immediate {
    fn from(b: bool) -> Immediate {
        Immediate::Bool(b)
    }
}

impl convert::From<i32> for Immediate {
    fn from(n: i32) -> Immediate {
        Immediate::Integer(n)
    }
}

impl convert::From<i32> for Object {
    fn from(n: i32) -> Object {
        Object::from_raw(ImmediateTag::Integer.tag(u64::from(n as u32)))
    }
}

impl convert::From<usize> for Immediate {
    fn from(n: usize) -> Immediate {
        Immediate::UnsignedInt(n)
    }
}

impl convert::From<usize> for Object {
    fn from(n: usize) -> Object {
        Object::from_raw(ImmediateTag::UnsignedInt.tag(n as u64))
    }
}

impl convert::From<bool> for Object {
    fn from(b: bool) -> Object {
        Object::from_raw(ImmediateTag::Bool.tag(b as u64))
    }
}

impl convert::From<SpecialMarker> for Object {
    fn from(s: SpecialMarker) -> Object {
        Object::from_raw(ImmediateTag::SpecialMarker.tag(u64::from(s as u32)))
    }
}

impl convert::From<SpecialMarker> for Immediate {
    fn from(s: SpecialMarker) -> Immediate {
        Immediate::SpecialMarker(s)
    }
}

impl fmt::Display for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Immediate::Bool(b) => {
                if b {
                    write!(f, "t")
                } else {
                    write!(f, "nil")
                }
            }
            Immediate::Integer(n) => write!(f, "{}", n),
            Immediate::UnsignedInt(n) => write!(f, "{}", n),
            Immediate::SpecialMarker(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Debug for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[immediate {}]", self)
    }
}
