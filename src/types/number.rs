use types::{self, reference, symbol, Object};
use types::conversions::*;
use std::{cmp, convert};
use symbol_lookup::make_symbol;

lazy_static! {
    static ref NUMBER_TYPE_NAME: symbol::SymRef = {
        make_symbol(b"number")
    };
}

#[derive(Clone, Copy)]
pub enum PhoebeNumber {
    Integer(i32),
    Float(f64),
}

fn fits_in_an_int(f: f64) -> bool {
    f <= f64::from(::std::i32::MAX) && f >= f64::from(::std::i32::MIN)
}

#[cfg_attr(feature = "cargo-clippy", allow(float_cmp))]
fn integerp(f: f64) -> bool {
    f.trunc() == f
}

fn try_flatten_float(f: f64) -> PhoebeNumber {
    if integerp(f) && fits_in_an_int(f) {
        PhoebeNumber::Integer(f as i32)
    } else {
        PhoebeNumber::Float(f)
    }
}

impl PhoebeNumber {
    pub fn try_flatten(self) -> Self {
        if let PhoebeNumber::Float(f) = self {
            try_flatten_float(f)
        } else {
            self
        }
    }
}

impl cmp::PartialEq for PhoebeNumber {
    fn eq(&self, rhs: &PhoebeNumber) -> bool {
        if let (Some(lhs), Some(rhs)) = (i32::maybe_from(*self), i32::maybe_from(*rhs)) {
            lhs == rhs
        } else {
            f64::from(*self) == f64::from(*rhs)
        }
    }
}

impl cmp::PartialOrd for PhoebeNumber {
    fn partial_cmp(&self, rhs: &PhoebeNumber) -> Option<cmp::Ordering> {
        if let (Some(lhs), Some(rhs)) = (i32::maybe_from(*self), i32::maybe_from(*rhs)) {
            lhs.partial_cmp(&rhs)
        } else {
            f64::from(*self).partial_cmp(&f64::from(*rhs))
        }
    }
    fn lt(&self, rhs: &PhoebeNumber) -> bool {
        if let (Some(lhs), Some(rhs)) = (i32::maybe_from(*self), i32::maybe_from(*rhs)) {
            lhs < rhs
        } else {
            f64::from(*self) < f64::from(*rhs)
        }
    }
    fn le(&self, rhs: &PhoebeNumber) -> bool {
        if let (Some(lhs), Some(rhs)) = (i32::maybe_from(*self), i32::maybe_from(*rhs)) {
            lhs <= rhs
        } else {
            f64::from(*self) <= f64::from(*rhs)
        }
    }
    fn gt(&self, rhs: &PhoebeNumber) -> bool {
        if let (Some(lhs), Some(rhs)) = (i32::maybe_from(*self), i32::maybe_from(*rhs)) {
            lhs > rhs
        } else {
            f64::from(*self) > f64::from(*rhs)
        }
    }
    fn ge(&self, rhs: &PhoebeNumber) -> bool {
        if let (Some(lhs), Some(rhs)) = (i32::maybe_from(*self), i32::maybe_from(*rhs)) {
            lhs >= rhs
        } else {
            f64::from(*self) >= f64::from(*rhs)
        }
    }
}

impl MaybeFrom<Object> for PhoebeNumber {
    fn maybe_from(obj: Object) -> Option<PhoebeNumber> {
        if let Some(f) = f64::maybe_from(obj) {
            Some(PhoebeNumber::Float(f))
        } else if let Some(n) = i32::maybe_from(obj) {
            Some(PhoebeNumber::Integer(n))
        } else if let Some(reference) = reference::Reference::maybe_from(obj) {
            PhoebeNumber::maybe_from(*reference)
        } else {
            None
        }
    }
}

impl FromObject for PhoebeNumber {
    /// pointer tagging is not meaningful for `PhoebeNumber` and
    /// `is_type` is overwritten, but we still have to provide a `Tag`
    /// and an `associated_tag`.
    type Tag = types::pointer_tagging::ObjectTag;
    fn associated_tag() -> types::pointer_tagging::ObjectTag {
        unreachable!()
    }

    fn type_name() -> symbol::SymRef {
        *NUMBER_TYPE_NAME
    }

    fn is_type(obj: Object) -> bool {
        f64::is_type(obj) || i32::is_type(obj)
    }
}

impl MaybeFrom<PhoebeNumber> for i32 {
    fn maybe_from(n: PhoebeNumber) -> Option<i32> {
        if let PhoebeNumber::Integer(n) = n {
            Some(n)
        } else {
            None
        }
    }
}

impl convert::From<PhoebeNumber> for f64 {
    fn from(n: PhoebeNumber) -> f64 {
        match n {
            PhoebeNumber::Float(f) => f,
            PhoebeNumber::Integer(i) => f64::from(i),
        }
    }
}

impl convert::From<f64> for PhoebeNumber {
    fn from(f: f64) -> PhoebeNumber {
        PhoebeNumber::Float(f)
    }
}

impl convert::From<i32> for PhoebeNumber {
    fn from(i: i32) -> PhoebeNumber {
        PhoebeNumber::Integer(i)
    }
}
