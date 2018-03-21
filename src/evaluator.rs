use types::{list, ExpandedObject, Object};
use symbol_lookup::lookup_symbol;
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

pub trait Evaluate {
    fn evaluate(&self) -> Result<Object, EvaluatorError>;
}

impl Evaluate for Object {
    fn evaluate(&self) -> Result<Object, EvaluatorError> {
        use stack::{pop, push};
        info!("Evaluating {}.", self);

        push(*self)?;

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

        let _popped = pop()?;
        debug_assert!(_popped == *self);

        res
    }
}

impl Evaluate for ExpandedObject {
    fn evaluate(&self) -> Result<Object, EvaluatorError> {
        Ok(match *self {
            ExpandedObject::Float(n) => Object::from(n),
            ExpandedObject::Immediate(i) => Object::from(i),
            ExpandedObject::Reference(r) => *r,
            ExpandedObject::Symbol(s) => Object::from(lookup_symbol(s)),
            ExpandedObject::Function(f) => Object::from(f),
            ExpandedObject::Cons(c) => unsafe { &*c }.evaluate()?,
            ExpandedObject::Namespace(n) => Object::from(n),
            ExpandedObject::HeapObject(h) => (*(unsafe { &*h })).evaluate()?,
        })
    }
}
