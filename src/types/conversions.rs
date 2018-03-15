use types::{Object, reference, symbol, pointer_tagging};
use types::pointer_tagging::PointerTag;

lazy_static! {
    static ref FLOAT_TYPE_NAME: symbol::SymRef = {
        ::symbol_lookup::make_symbol(b"float")
    };
    static ref INTEGER_TYPE_NAME: symbol::SymRef = {
        ::symbol_lookup::make_symbol(b"integer")
    };
    static ref BOOL_TYPE_NAME: symbol::SymRef = {
        ::symbol_lookup::make_symbol(b"booleanr")
    };
}

pub trait MaybeFrom<T: Sized>: Sized {
    fn maybe_from(t: T) -> Option<Self>;
}

pub trait FromUnchecked<T: Sized>: Sized {
    unsafe fn from_unchecked(obj: T) -> Self;
}

pub trait MaybeInto<T: Sized>: Sized {
    fn maybe_into(self) -> Option<T>;
}

pub trait IntoUnchecked<T: Sized>: Sized {
    unsafe fn into_unchecked(self) -> T;
}

pub trait FromObject {
    type Tag: pointer_tagging::PointerTag;
    fn associated_tag() -> Self::Tag;
    fn type_name() -> symbol::SymRef;
    fn is_type(obj: Object) -> bool {
        Self::associated_tag().is_of_type(obj.0)
    }
    fn derefs_to(obj: Object) -> bool {
        Self::is_type(obj)
            || if let Some(r) = reference::Reference::maybe_from(obj) {
                Self::derefs_to(*r)
            } else {
                false
            }
    }
}

impl<T> MaybeFrom<Object> for T
where T: FromUnchecked<Object> + FromObject {
    default fn maybe_from(obj: Object) -> Option<T> {
        if <T as FromObject>::is_type(obj) {
            Some(unsafe { T::from_unchecked(obj) })
        } else if let Some(r) = reference::Reference::maybe_from(obj) {
            T::maybe_from(*r)
        } else {
            None
        }
    }
}

impl<T, O> IntoUnchecked<T> for O
where T: FromUnchecked<O> {
    unsafe fn into_unchecked(self) -> T {
        T::from_unchecked(self)
    }
}

impl<T, O> MaybeInto<T> for O
where T: MaybeFrom<O> {
    fn maybe_into(self) -> Option<T> {
        T::maybe_from(self)
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
    fn type_name() -> symbol::SymRef {
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
    fn type_name() -> symbol::SymRef {
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
    fn type_name() -> symbol::SymRef {
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
    fn type_name() -> symbol::SymRef {
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
    fn type_name() -> symbol::SymRef {
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
    fn type_name() -> symbol::SymRef {
        *INTEGER_TYPE_NAME
    }
}
