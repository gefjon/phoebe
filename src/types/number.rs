use prelude::*;
use std::{cmp, convert, ops};
use symbol_lookup::make_symbol;
use types::pointer_tagging;

lazy_static! {
    static ref NUMBER_TYPE_NAME: GcRef<Symbol> = { make_symbol(b"number") };
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
    pub fn recip(self) -> Self {
        let recip = 1.0 / (f64::from(self));
        try_flatten_float(recip)
    }
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

impl ops::Add for PhoebeNumber {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        if let (Some(l), Some(r)) = (i32::maybe_from(self), i32::maybe_from(other)) {
            (l + r).into()
        } else {
            (f64::from(self) + f64::from(other)).into()
        }
    }
}

impl ops::AddAssign for PhoebeNumber {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl ops::Sub for PhoebeNumber {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        if let (Some(l), Some(r)) = (i32::maybe_from(self), i32::maybe_from(other)) {
            (l - r).into()
        } else {
            (f64::from(self) - f64::from(other)).into()
        }
    }
}

impl ops::SubAssign for PhoebeNumber {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl ops::Mul for PhoebeNumber {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        if let (Some(l), Some(r)) = (i32::maybe_from(self), i32::maybe_from(other)) {
            (l * r).into()
        } else {
            (f64::from(self) * f64::from(other)).into()
        }
    }
}

impl ops::MulAssign for PhoebeNumber {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl ops::Div for PhoebeNumber {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self::from(f64::from(self) / f64::from(other)).try_flatten()
    }
}

impl ops::DivAssign for PhoebeNumber {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl ops::Neg for PhoebeNumber {
    type Output = Self;
    fn neg(self) -> Self {
        if let Some(n) = i32::maybe_from(self) {
            PhoebeNumber::from(-n)
        } else {
            PhoebeNumber::from(-(f64::from(self)))
        }
    }
}

impl MaybeFrom<Object> for PhoebeNumber {
    fn maybe_from(obj: Object) -> Option<PhoebeNumber> {
        if let Some(f) = f64::maybe_from(obj) {
            Some(PhoebeNumber::Float(f))
        } else if let Some(n) = i32::maybe_from(obj) {
            Some(PhoebeNumber::Integer(n))
        } else if let Some(reference) = Reference::maybe_from(obj) {
            PhoebeNumber::maybe_from(*reference)
        } else {
            None
        }
    }
    fn try_convert_from(obj: Object) -> Result<PhoebeNumber, ConversionError> {
        if let Some(t) = PhoebeNumber::maybe_from(obj) {
            Ok(t)
        } else {
            Err(ConversionError::wanted(PhoebeNumber::type_name()))
        }
    }
}

impl FromObject for PhoebeNumber {
    /// pointer tagging is not meaningful for `PhoebeNumber` and
    /// `is_type` is overwritten, but we still have to provide a `Tag`
    /// and an `associated_tag`.
    type Tag = pointer_tagging::ObjectTag;
    fn associated_tag() -> pointer_tagging::ObjectTag {
        unreachable!()
    }

    fn type_name() -> GcRef<Symbol> {
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
    fn try_convert_from(obj: PhoebeNumber) -> Result<i32, ConversionError> {
        if let Some(t) = i32::maybe_from(obj) {
            Ok(t)
        } else {
            Err(ConversionError::wanted(i32::type_name()))
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

impl convert::From<PhoebeNumber> for Object {
    fn from(n: PhoebeNumber) -> Object {
        if let Some(n) = i32::maybe_from(n) {
            Object::from(n)
        } else {
            Object::from(f64::from(n))
        }
    }
}
