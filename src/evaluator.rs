use types::{list, ExpandedObject, Object};
use types::reference::Reference;
use symbol_lookup::{lookup_symbol, UnboundSymbolError};
use gc::gc_maybe_pass;
use stack::{StackOverflowError, StackUnderflowError};
use std::convert;
use types::conversions::ConversionError;

#[derive(Fail, Debug)]
pub enum EvaluatorError {
    #[fail(display = "{}", _0)]
    StackOverflow(StackOverflowError),
    #[fail(display = "{}", _0)]
    StackUnderflow(StackUnderflowError),
    #[fail(display = "The count {} is not compatible with the arglist {}", found, arglist)]
    BadArgCount { arglist: list::List, found: usize },
    #[fail(display = "{}", _0)]
    TypeError(ConversionError),
    #[fail(display = "Found an improperly-terminated list where a proper one was expected")]
    ImproperList,
    #[fail(display = "Attempt to create a reference has failed")]
    CannotBeReferenced,
    #[fail(display = "{}", _0)]
    UnboundSymbol(UnboundSymbolError),
}

unsafe impl Sync for EvaluatorError {}
unsafe impl Send for EvaluatorError {}

impl EvaluatorError {
    pub fn bad_args_count(arglist: list::List, found: usize) -> Self {
        EvaluatorError::BadArgCount { arglist, found }
    }
}

impl convert::From<ConversionError> for EvaluatorError {
    fn from(e: ConversionError) -> EvaluatorError {
        EvaluatorError::TypeError(e)
    }
}

impl convert::From<StackOverflowError> for EvaluatorError {
    fn from(e: StackOverflowError) -> EvaluatorError {
        EvaluatorError::StackOverflow(e)
    }
}

impl convert::From<StackUnderflowError> for EvaluatorError {
    fn from(e: StackUnderflowError) -> EvaluatorError {
        EvaluatorError::StackUnderflow(e)
    }
}

impl convert::From<UnboundSymbolError> for EvaluatorError {
    fn from(e: UnboundSymbolError) -> EvaluatorError {
        EvaluatorError::UnboundSymbol(e)
    }
}

pub trait Evaluate {
    fn eval_to_reference(&self) -> Result<Reference, EvaluatorError> {
        Err(EvaluatorError::CannotBeReferenced)
    }
    fn evaluate(&self) -> Result<Object, EvaluatorError>;
}

impl Evaluate for Object {
    fn evaluate(&self) -> Result<Object, EvaluatorError> {
        info!("Evaluating {}.", self);

        let res = ExpandedObject::from(*self).evaluate();
        if !res.is_err() {
            debug!("Not an error; might garbage collect.");
            // it is only safe to garbage-collect if `evaluate`
            // returned `Ok` - `EvaluatorError`s can contain
            // references to garbage-collected objects, but since
            // `EvaluatorError`s are not themselves garbage-collected
            // (yet), those objects may be deallocated prematurely.
            gc_maybe_pass();
        }

        res
    }
    fn eval_to_reference(&self) -> Result<Reference, EvaluatorError> {
        let res = ExpandedObject::from(*self).eval_to_reference();
        if !res.is_err() {
            debug!("Not an error; might garbage-collect.");
            gc_maybe_pass();
        }
        res
    }
}

impl Evaluate for ExpandedObject {
    fn evaluate(&self) -> Result<Object, EvaluatorError> {
        Ok(match *self {
            ExpandedObject::Float(n) => Object::from(n),
            ExpandedObject::Immediate(i) => Object::from(i),
            ExpandedObject::Reference(r) => *r,
            ExpandedObject::Symbol(s) => *(lookup_symbol(s)?),
            ExpandedObject::Function(f) => Object::from(f),
            ExpandedObject::Cons(c) => unsafe { &*c }.evaluate()?,
            ExpandedObject::Namespace(n) => Object::from(n),
            ExpandedObject::HeapObject(h) => (*(unsafe { &*h })).evaluate()?,
        })
    }
    fn eval_to_reference(&self) -> Result<Reference, EvaluatorError> {
        match *self {
            ExpandedObject::Float(_)
            | ExpandedObject::Immediate(_)
            | ExpandedObject::Function(_)
            | ExpandedObject::Namespace(_) => Err(EvaluatorError::CannotBeReferenced),
            ExpandedObject::Reference(r) => Ok(r),
            ExpandedObject::Symbol(s) => Ok(lookup_symbol(s)?),
            ExpandedObject::Cons(c) => unsafe { &*c }.eval_to_reference(),
            ExpandedObject::HeapObject(h) => (*(unsafe { &*h })).eval_to_reference(),
        }
    }
}
