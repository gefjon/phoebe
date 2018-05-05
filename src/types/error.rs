use prelude::*;
use stack::{ArgIndexError, StackOverflowError, StackUnderflowError};
use std::convert;
use symbol_lookup::UnboundSymbolError;
use types::conversions::ConversionError;
use types::pointer_tagging::{ObjectTag, PointerTag};

lazy_static! {
    static ref ERROR_TYPE_NAME: GcRef<Symbol> = { symbol_lookup::make_symbol(b"error") };
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
/// This tag is only allowed to take 3 bits - the low three bits of
/// any `*mut Error` will always `=== 0b000` because of `Error`'s
/// alignment. It is marked `#[repr(u8)]` because `u3` is not a type.
pub enum ErrorTag {
    /// This mask represents a thrown `Error` - one that should bubble
    /// up the stack until handle.
    Signaling,
    /// This mask represents a handled `Error` - one which can be
    /// manipulated like any other `Object`. `catch-error` (or
    /// similar) converts a `Signaling` `Error` into a `Quiet` one.
    Quiet,
}

impl convert::From<ErrorTag> for u64 {
    fn from(t: ErrorTag) -> u64 {
        t as u64
    }
}

impl PointerTag for ErrorTag {
    fn mask_bits() -> u64 {
        0b111
    }
    fn parent_mask() -> u64 {
        ObjectTag::parent_mask() ^ ObjectTag::mask_bits()
    }
    fn parent_tag() -> u64 {
        ObjectTag::Error.tag(0)
    }
}

#[derive(Fail, Debug)]
#[fail(display = "{}", error)]
pub struct Error {
    gc_marking: GcMark,
    error: EvaluatorError,
}

impl Error {
    pub fn type_error(wanted_type: GcRef<Symbol>) -> GcRef<Error> {
        EvaluatorError::TypeError(ConversionError::wanted(wanted_type)).into()
    }
    pub fn improper_list() -> GcRef<Error> {
        EvaluatorError::ImproperList.into()
    }
    pub fn cannot_be_referenced() -> GcRef<Error> {
        EvaluatorError::CannotBeReferenced.into()
    }
    pub fn user(name: GcRef<Symbol>, body: Object) -> GcRef<Error> {
        EvaluatorError::user(name, body).into()
    }
    pub fn name(&self) -> GcRef<Symbol> {
        symbol_lookup::make_symbol(match self.error {
            EvaluatorError::StackOverflow(_) => b"stack-overflow-error",
            EvaluatorError::StackUnderflow(_) => b"stack-underflow-error",
            EvaluatorError::BadArgCount { .. } => b"arg-count-error",
            EvaluatorError::TypeError(_) => b"type-error",
            EvaluatorError::ImproperList => b"improper-list-error",
            EvaluatorError::CannotBeReferenced => b"not-a-reference-error",
            EvaluatorError::UnboundSymbol(_) => b"unbound-symbol-error",
            EvaluatorError::UnaccompaniedKey { .. } => b"unaccompanied-key-error",
            EvaluatorError::ArgIndex(_) => b"arg-out-of-bounds-error",
            EvaluatorError::User { name, .. } => {
                return name;
            }
        })
    }
}

#[derive(Fail, Debug)]
/// Represents the different ways that evaluation can fail. In the
/// future, when Phoebe has language-level error handling as a
/// feature, there will be some language way to interact with this
/// type, as well as a variant which contains an `Object`.
pub enum EvaluatorError {
    #[fail(display = "{}", _0)]
    StackOverflow(StackOverflowError),

    #[fail(display = "{}", _0)]
    StackUnderflow(StackUnderflowError),

    #[fail(display = "The count {} is not compatible with the arglist {}", found, arglist)]
    /// Functions which are passed incompatible numbers of arguments
    /// signal this error.
    BadArgCount { arglist: List, found: usize },

    #[fail(display = "{}", _0)]
    TypeError(ConversionError),

    #[fail(display = "Found an improperly-terminated list where a proper one was expected")]
    /// Denotes an improperly terminated or looped list where a
    /// `nil`-terminated list was expected.
    ImproperList,

    #[fail(display = "Attempt to create a reference has failed")]
    /// Calls to `Evaluate::eval_to_reference` which do not produce a
    /// reference result in this error.
    CannotBeReferenced,

    #[fail(display = "{}", _0)]
    UnboundSymbol(UnboundSymbolError),

    #[fail(
        display = "The key {} did not have an accompanying symbol when parsing key arguments.", key
    )]
    UnaccompaniedKey { key: GcRef<Symbol> },

    #[fail(display = "{}", _0)]
    ArgIndex(ArgIndexError),

    #[fail(display = "{}: {}", name, body)]
    User { name: GcRef<Symbol>, body: Object },
}

impl convert::From<EvaluatorError> for Error {
    fn from(error: EvaluatorError) -> Error {
        Error {
            gc_marking: GcMark::default(),
            error,
        }
    }
}

impl EvaluatorError {
    pub fn bad_args_count(arglist: List, found: usize) -> Self {
        EvaluatorError::BadArgCount { arglist, found }
    }
    pub fn user(name: GcRef<Symbol>, body: Object) -> EvaluatorError {
        EvaluatorError::User { name, body }
    }
}

impl convert::From<ArgIndexError> for EvaluatorError {
    fn from(e: ArgIndexError) -> Self {
        EvaluatorError::ArgIndex(e)
    }
}

impl convert::From<ArgIndexError> for Error {
    fn from(e: ArgIndexError) -> Self {
        let e = EvaluatorError::from(e);
        e.into()
    }
}

impl convert::From<ArgIndexError> for GcRef<Error> {
    fn from(e: ArgIndexError) -> Self {
        Error::allocate(e.into())
    }
}

impl convert::From<ConversionError> for EvaluatorError {
    fn from(e: ConversionError) -> Self {
        EvaluatorError::TypeError(e)
    }
}

impl convert::From<ConversionError> for Error {
    fn from(e: ConversionError) -> Self {
        let e = EvaluatorError::from(e);
        e.into()
    }
}

impl convert::From<ConversionError> for GcRef<Error> {
    fn from(e: ConversionError) -> Self {
        Error::allocate(e.into())
    }
}

impl convert::From<StackOverflowError> for EvaluatorError {
    fn from(e: StackOverflowError) -> Self {
        EvaluatorError::StackOverflow(e)
    }
}

impl convert::From<StackOverflowError> for Error {
    fn from(e: StackOverflowError) -> Self {
        let e = EvaluatorError::from(e);
        e.into()
    }
}

impl convert::From<StackOverflowError> for GcRef<Error> {
    fn from(e: StackOverflowError) -> Self {
        Error::allocate(e.into())
    }
}

impl convert::From<StackUnderflowError> for EvaluatorError {
    fn from(e: StackUnderflowError) -> Self {
        EvaluatorError::StackUnderflow(e)
    }
}

impl convert::From<StackUnderflowError> for Error {
    fn from(e: StackUnderflowError) -> Self {
        let e = EvaluatorError::from(e);
        e.into()
    }
}

impl convert::From<StackUnderflowError> for GcRef<Error> {
    fn from(e: StackUnderflowError) -> Self {
        Error::allocate(e.into())
    }
}

impl convert::From<UnboundSymbolError> for EvaluatorError {
    fn from(e: UnboundSymbolError) -> Self {
        EvaluatorError::UnboundSymbol(e)
    }
}

impl convert::From<UnboundSymbolError> for Error {
    fn from(e: UnboundSymbolError) -> Self {
        let e = EvaluatorError::from(e);
        e.into()
    }
}

impl convert::From<UnboundSymbolError> for GcRef<Error> {
    fn from(e: UnboundSymbolError) -> Self {
        Error::allocate(e.into())
    }
}

impl convert::From<EvaluatorError> for GcRef<Error> {
    fn from(e: EvaluatorError) -> Self {
        Error::allocate(e)
    }
}

impl convert::From<EvaluatorError> for Object {
    fn from(e: EvaluatorError) -> Self {
        Object::loud_error(e.into())
    }
}

impl convert::From<GcRef<Error>> for Object {
    /// This method builds a *Signaling* error. For a non-signaling
    /// error, use `Object::quiet_error`. Choose to use
    /// `Object::loud_error` rather than this whenever possible, as it
    /// is more expressive.
    fn from(e: GcRef<Error>) -> Object {
        Object::loud_error(e)
    }
}

impl FromUnchecked<Object> for GcRef<Error> {
    unsafe fn from_unchecked(obj: Object) -> GcRef<Error> {
        GcRef::from_ptr(if ErrorTag::Signaling.is_of_type(obj.into_raw()) {
            ErrorTag::Signaling.untag(obj.into_raw())
        } else {
            ErrorTag::Quiet.untag(obj.into_raw())
        } as *mut Error)
    }
}

impl FromObject for GcRef<Error> {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Error
    }
    fn type_name() -> GcRef<Symbol> {
        *ERROR_TYPE_NAME
    }
}

impl GarbageCollected for Error {
    type ConvertFrom = EvaluatorError;
    fn alloc_one_and_initialize(o: EvaluatorError) -> ::std::ptr::NonNull<Self> {
        use std::{
            alloc::{Alloc, Global}, ptr,
        };
        let nn = Global.alloc_one().unwrap();
        let p = nn.as_ptr();
        unsafe { ptr::write(p, o.into()) };
        nn
    }
    fn my_marking(&self) -> &GcMark {
        &self.gc_marking
    }
    fn gc_mark_children(&mut self, mark: usize) {
        match self.error {
            EvaluatorError::BadArgCount { arglist, .. } => {
                if let Some(c) = <GcRef<Cons>>::maybe_from(arglist) {
                    c.gc_mark(mark);
                }
            }
            EvaluatorError::TypeError(ConversionError { wanted_type, .. }) => {
                wanted_type.gc_mark(mark)
            }
            EvaluatorError::UnboundSymbol(UnboundSymbolError { sym, .. }) => sym.gc_mark(mark),
            EvaluatorError::UnaccompaniedKey { key, .. } => key.gc_mark(mark),
            EvaluatorError::User { name, body } => {
                name.gc_mark(mark);
                body.gc_mark(mark);
            }
            _ => (),
        }
    }
}

unsafe impl Sync for Error {}
unsafe impl Send for Error {}
