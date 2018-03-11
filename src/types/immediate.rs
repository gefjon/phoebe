use types::pointer_tagging::{PointerTag, ObjectTag};
use std::{convert, fmt};
use super::Object;
use super::conversions::*;

const IMMEDIATE_TAG_MASK: u64 = 0xffff << 32;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Immediate {
    Bool(bool),
    Integer(i32),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ImmediateTag {
    Bool,
    Integer,
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
    fn type_name() -> *const super::symbol::Symbol {
        unimplemented!()
    }
}

impl convert::From<Immediate> for Object {
    fn from(i: Immediate) -> Object {
        Object(match i {
            Immediate::Bool(b) => ImmediateTag::Bool.tag(b as u64),
            Immediate::Integer(n) => ImmediateTag::Integer.tag(n as u32 as u64),
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

impl fmt::Display for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Immediate::Bool(b) => if b { write!(f, "t") } else { write!(f, "nil") },
            &Immediate::Integer(n) => write!(f, "{}", n),
        }
    }
}

impl fmt::Debug for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[immediate {}]", self)
    }
}
            
