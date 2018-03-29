use prelude::*;
use types::pointer_tagging::{self, PointerTag};

lazy_static! {
    static ref FLOAT_TYPE_NAME: GcRef<Symbol> = {
        symbol_lookup::make_symbol(b"float")
    };

    // This could be changed to `fixnum` in the future if Phoebe gets
    // a `bignum` type and wants to use `integer` for the set of all
    // `fixnum`s and `bignum`s.
    static ref INTEGER_TYPE_NAME: GcRef<Symbol> = {
        symbol_lookup::make_symbol(b"integer")
    };
    static ref BOOL_TYPE_NAME: GcRef<Symbol> = {
        symbol_lookup::make_symbol(b"boolean")
    };
}

#[derive(Fail, Debug)]
#[fail(display = "Expected a value of type {}.", wanted_type)]
pub struct ConversionError {
    pub wanted_type: GcRef<Symbol>,
}

impl ConversionError {
    pub fn wanted(wanted_type: GcRef<Symbol>) -> ConversionError {
        ConversionError { wanted_type }
    }
}

/// This trait is analogous to `std::convert::From` and `TryFrom`,
/// only it returns an `Option<Self>` instead of a `Self` or
/// `Result<Self>`. I find it more idiomatic to use an `Option` to
/// denote conversions between subtypes than a `Result` - failure to
/// convert an `Object` into a type is not an error condition.
pub trait MaybeFrom<T: Sized>: Sized {
    fn maybe_from(t: T) -> Option<Self>;
    fn try_convert_from(t: T) -> Result<Self, ConversionError>;
}

/// This trait is analogous to `std::convert::From` but marked
/// `unsafe`. The idea behind this is to expose a method for the raw
/// tagging and untagging of `Object`s which can be called by
/// `MaybeFrom` after type checking, or in cases where we are
/// absolutely sure we have an object of the correct type.
pub trait FromUnchecked<T: Sized>: Sized {
    unsafe fn from_unchecked(obj: T) -> Self;
}

/// The companion trait to `MaybeFrom` - automatically derived for
/// `MaybeFrom` types.
pub trait MaybeInto<T: Sized>: Sized {
    fn maybe_into(self) -> Option<T>;
    fn try_convert_into(self) -> Result<T, ConversionError>;
}

/// The companion trait to `FromUnchecked` - automatically derived for
/// `FromUnchecked` types.
pub trait IntoUnchecked<T: Sized>: Sized {
    unsafe fn into_unchecked(self) -> T;
}

/// This trait encapsulates several utility functions related to
/// converting from `Object`.
pub trait FromObject {
    type Tag: pointer_tagging::PointerTag;
    fn associated_tag() -> Self::Tag;

    /// Returns a `SymRef` to the name of this type. This is coupled
    /// with a `lazy_static` which creates the `SymRef` and holds onto
    /// it; check the implementation for `f64`, `i32` or `bool` for
    /// examples.
    fn type_name() -> GcRef<Symbol>;

    /// Returns `true` iff `obj` is a correctly tagged `Self`. Note
    /// that being correctly tagged does not necessarily imply being a
    /// valid value - this function will not check that by-reference
    /// types are non-null or that they point to a valid instance.
    fn is_type(obj: Object) -> bool {
        Self::associated_tag().is_of_type(obj.0)
    }

    /// A generalization of `is_type`; also returns `true` if `obj` is
    /// a `Reference` pointing to a correctly tagged `Self`.
    fn derefs_to(obj: Object) -> bool {
        Self::is_type(obj) || if let Some(r) = Reference::maybe_from(obj) {
            Self::derefs_to(*r)
        } else {
            false
        }
    }
}

impl<T> MaybeFrom<Object> for T
where
    T: FromUnchecked<Object> + FromObject,
{
    default fn maybe_from(obj: Object) -> Option<T> {
        if <T as FromObject>::is_type(obj) {
            Some(unsafe { T::from_unchecked(obj) })
        } else if let Some(r) = Reference::maybe_from(obj) {
            T::maybe_from(*r)
        } else {
            None
        }
    }
    default fn try_convert_from(obj: Object) -> Result<T, ConversionError> {
        if let Some(t) = T::maybe_from(obj) {
            Ok(t)
        } else {
            Err(ConversionError::wanted(T::type_name()))
        }
    }
}

impl<T, O> IntoUnchecked<T> for O
where
    T: FromUnchecked<O>,
{
    unsafe fn into_unchecked(self) -> T {
        T::from_unchecked(self)
    }
}

impl<T, O> MaybeInto<T> for O
where
    T: MaybeFrom<O>,
{
    fn maybe_into(self) -> Option<T> {
        T::maybe_from(self)
    }
    fn try_convert_into(self) -> Result<T, ConversionError> {
        T::try_convert_from(self)
    }
}

impl<T> FromUnchecked<Object> for *const T
where
    *mut T: FromUnchecked<Object>,
{
    unsafe fn from_unchecked(obj: Object) -> *const T {
        <*mut T>::from_unchecked(obj) as *const T
    }
}

impl<T> FromUnchecked<Object> for &'static T
where
    *const T: FromUnchecked<Object>,
{
    unsafe fn from_unchecked(obj: Object) -> &'static T {
        &*(<*const T>::from_unchecked(obj))
    }
}

impl<T> FromUnchecked<Object> for &'static mut T
where
    *mut T: FromUnchecked<Object>,
{
    unsafe fn from_unchecked(obj: Object) -> &'static mut T {
        &mut *(<*mut T>::from_unchecked(obj))
    }
}

impl<T> FromObject for *const T
where
    *mut T: FromObject,
{
    type Tag = <*mut T as FromObject>::Tag;
    fn associated_tag() -> Self::Tag {
        <*mut T as FromObject>::associated_tag()
    }
    fn type_name() -> GcRef<Symbol> {
        <*mut T>::type_name()
    }
}

impl<T> FromObject for &'static T
where
    *const T: FromObject,
{
    type Tag = <*const T as FromObject>::Tag;
    fn associated_tag() -> Self::Tag {
        <*const T as FromObject>::associated_tag()
    }
    fn type_name() -> GcRef<Symbol> {
        <*const T>::type_name()
    }
}

impl<T> FromObject for &'static mut T
where
    *mut T: FromObject,
{
    type Tag = <*mut T as FromObject>::Tag;
    fn associated_tag() -> Self::Tag {
        <*mut T as FromObject>::associated_tag()
    }
    fn type_name() -> GcRef<Symbol> {
        <*mut T>::type_name()
    }
}

impl FromUnchecked<Object> for f64 {
    unsafe fn from_unchecked(obj: Object) -> f64 {
        f64::from_bits(obj.0)
    }
}

impl FromObject for f64 {
    type Tag = super::pointer_tagging::Floatp;
    fn associated_tag() -> super::pointer_tagging::Floatp {
        super::pointer_tagging::Floatp::Float
    }
    fn type_name() -> GcRef<Symbol> {
        *FLOAT_TYPE_NAME
    }
}

impl FromUnchecked<Object> for bool {
    unsafe fn from_unchecked(obj: Object) -> bool {
        use types::immediate::ImmediateTag;

        ImmediateTag::Bool.untag(obj.0) != 0
    }
}

impl FromObject for bool {
    type Tag = super::immediate::ImmediateTag;
    fn associated_tag() -> super::immediate::ImmediateTag {
        super::immediate::ImmediateTag::Bool
    }
    fn type_name() -> GcRef<Symbol> {
        *BOOL_TYPE_NAME
    }
}

impl FromUnchecked<Object> for i32 {
    unsafe fn from_unchecked(obj: Object) -> i32 {
        use types::immediate::ImmediateTag;

        (ImmediateTag::Integer.untag(obj.0) as u32) as i32
    }
}

impl FromObject for i32 {
    type Tag = super::immediate::ImmediateTag;
    fn associated_tag() -> super::immediate::ImmediateTag {
        super::immediate::ImmediateTag::Integer
    }
    fn type_name() -> GcRef<Symbol> {
        *INTEGER_TYPE_NAME
    }
}
