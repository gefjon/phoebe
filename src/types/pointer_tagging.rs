use std::{convert};

pub const POINTER_TAGGING_MASK: u64 = 0b111;
pub const OBJECT_TAG_MASK: u64 = 0b1111 << 48;
pub const NAN_MASK: u64 = 0b111_1111_1111 << 52;

pub trait PointerTag: Into<u64> + Copy {
    fn mask_bits() -> u64;

    /// For nested tagged pointer types (`ImmediateTag` is a child of
    /// `ObjectTag::Immediate, for example`), `parent_tag` represents
    /// a tag which all values of `Self` will have, regardless of
    /// which variant.
    fn parent_tag() -> u64;
    fn parent_mask() -> u64;
    fn is_correct_parent(ptr: u64) -> bool {
        (ptr & Self::parent_mask()) == Self::parent_tag()
    }
    fn tag_bits_match(self, ptr: u64) -> bool {
        (ptr & Self::mask_bits()) == self.into()
    }
    fn val_will_fit(val: u64) -> bool {
        (val & Self::parent_mask()) == 0
            && (val & Self::mask_bits()) == 0
    }
    fn tag(self, ptr: u64) -> u64 {
        debug_assert!((ptr & Self::mask_bits()) == 0);
        self.into() ^ ptr ^ Self::parent_tag()
    }
    fn is_of_type(self, ptr: u64) -> bool {
        Self::is_correct_parent(ptr)
            && self.tag_bits_match(ptr)
    }
    fn untag(self, ptr: u64) -> u64 {
        debug_assert!(self.is_of_type(ptr));
        ptr & !(self.into())
    }
}

/// There could conceivably be up to 16 variants of `ObjectTag`, as
/// `Object` leaves 4 bits of tag between the NaN marker and the 48
/// bit integer immediate.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u64)]
pub enum ObjectTag {
    Cons,
    Symbol,
    // String,
    // Function,
    // Error,
    // Namespace,
    Immediate,
    Reference,
}

impl convert::From<ObjectTag> for u64 {
    fn from(t: ObjectTag) -> u64 {
        (t as u64) << 48
    }
}

impl PointerTag for ObjectTag {
    fn mask_bits() -> u64 {
        OBJECT_TAG_MASK
    }
    fn parent_tag() -> u64 {
        NAN_MASK
    }
    fn parent_mask() -> u64 {
        NAN_MASK
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u64)]
pub enum Floatp {
    NaN,
    Float,
}

impl convert::From<Floatp> for u64 {
    fn from(_: Floatp) -> u64 {
        unreachable!()
    }
}

impl PointerTag for Floatp {
    fn mask_bits() -> u64 {
        0b111_1111_1111 << 52
    }
    fn parent_tag() -> u64 {
        0
    }
    fn parent_mask() -> u64 {
        0
    }
    fn is_correct_parent(_: u64) -> bool {
        true
    }
    fn tag_bits_match(self, ptr: u64) -> bool {
        match self {
            Floatp::NaN => (ptr & Self::mask_bits()) == Self::mask_bits(),
            Floatp::Float => (ptr & Self::mask_bits()) != Self::mask_bits(),
        }
    }
    fn val_will_fit(_: u64) -> bool {
        true
    }
    fn tag(self, ptr: u64) -> u64 {
        match self {
            Floatp::NaN => ptr & Self::mask_bits(),
            Floatp::Float => ptr,
        }
    }
    fn untag(self, ptr: u64) -> u64 {
        debug_assert!(self.is_of_type(ptr));
        match self {
            Floatp::NaN => ptr & !Self::mask_bits(),
            Floatp::Float => ptr,
        }
    }
}
