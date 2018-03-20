use types::{Object, ExpandedObject, list};
use symbol_lookup::{lookup_symbol};
use gc::gc_maybe_pass;
use stack::{StackOverflowError, StackUnderflowError};
use std::convert;

#[derive(Fail, Debug)]
pub enum EvaluatorError {
    #[fail(display = "{}", _0)]
    StackOverflow(StackOverflowError),
    #[fail(display = "{}", _0)]
    StackUnderflow(StackUnderflowError),
    #[fail(display = "The count {} is not compatible with the arglist {}", found, arglist)]
    BadArgCount {
        arglist: list::List,
        found: usize,
    }
}

unsafe impl Sync for EvaluatorError {}
unsafe impl Send for EvaluatorError {}

impl EvaluatorError {
    pub fn bad_args_count(arglist: list::List, found: usize) -> Self {
        EvaluatorError::BadArgCount {
            arglist,
            found,
        }
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
        let res = ExpandedObject::from(*self).evaluate();
        gc_maybe_pass();
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
