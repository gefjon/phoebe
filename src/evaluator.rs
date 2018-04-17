//! This module contains the framework for evaluation, which manifests
//! as:
//!
//! * the trait `Evaluate`
//! * the enum `EvaluatorError`

use gc::GcRef;
use prelude::*;
use stack::{ArgIndexError, StackOverflowError, StackUnderflowError};
use std::convert;
use symbol_lookup::UnboundSymbolError;
use types::conversions::ConversionError;
use types::reference::Reference;
use types::symbol::Symbol;
use types::{list, ExpandedObject, Object};

pub unsafe fn eval_from_stack() -> Result<(), EvaluatorError> {
    let to_eval = stack::nth_arg(0)?;
    let res = (*to_eval).evaluate()?;
    stack::close_stack_frame_and_return(res);
    Ok(())
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
    BadArgCount { arglist: list::List, found: usize },

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
}

unsafe impl Sync for EvaluatorError {}
unsafe impl Send for EvaluatorError {}

impl EvaluatorError {
    pub fn bad_args_count(arglist: list::List, found: usize) -> Self {
        EvaluatorError::BadArgCount { arglist, found }
    }
}

impl convert::From<ArgIndexError> for EvaluatorError {
    fn from(e: ArgIndexError) -> EvaluatorError {
        EvaluatorError::ArgIndex(e)
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
    /// This method is called by `setf`. Because it is usually
    /// undesirable to return references (it causes strange errors in
    /// cases such as:
    ///
    /// ```lisp,text
    /// (defvar x 1)
    /// => 1
    /// (defun return-x ()
    ///   x)
    /// => [function return-x]
    /// (list
    ///   (return-x)
    ///   (progn
    ///     (setf x 2)
    ///     (return-x)))
    /// => (2 2)
    /// ```
    ///
    /// The default implementation of this method returns
    /// `Err(CannotBeReferenced)` unconditionally, so types which
    /// cannot be evaluated to a reference need not worry about
    /// implementing it.
    ///
    /// NOTE: evaluating special forms, like `cond`, `defvar`, `setf`,
    /// etc., incorrectly do not recurse on `eval_to_reference` and
    /// instead always call `evaluate`.
    ///
    /// NOTE: In cases where a source function returns one of its
    /// arguments, `eval_to_reference` can cause it to return a
    /// reference to a dead piece of the stack, which is UB and may
    /// segfault in threaded contexts. eg:
    ///
    /// ```lisp,text
    /// (defun returns-arg (foo)
    ///   foo)
    /// ```
    fn eval_to_reference(&self) -> Result<Reference, EvaluatorError> {
        Err(EvaluatorError::CannotBeReferenced)
    }
    fn evaluate(&self) -> Result<Object, EvaluatorError>;
}

impl Evaluate for Object {
    /// `evaluate`, like most operations on `Object`s, involves
    /// deconstructing `self` into an `ExpandedObject` and then
    /// calling `evaluate` on that.
    fn evaluate(&self) -> Result<Object, EvaluatorError> {
        info!("Evaluating {}.", self);

        ExpandedObject::from(*self).evaluate()
    }
    fn eval_to_reference(&self) -> Result<Reference, EvaluatorError> {
        info!("Evaluating {} to reference.", self);

        ExpandedObject::from(*self).eval_to_reference()
    }
}

impl Evaluate for ExpandedObject {
    /// Floats, `Immediate`s, `Function`s and `Namespace`s are all
    /// self-evaluating. `Reference`s evaluate to the value they
    /// dereference to. `HeapObject`s evaluate by dereferencing and
    /// evaluating themselves. `Symbol`s are looked up. `Cons`es are
    /// the only `Object`s with a serious, beefy `evaluate`
    /// implementation.
    fn evaluate(&self) -> Result<Object, EvaluatorError> {
        Ok(match *self {
            ExpandedObject::Float(n) => Object::from(n),
            ExpandedObject::Immediate(i) => Object::from(i),
            ExpandedObject::Reference(ref r) => **r,
            ExpandedObject::Symbol(s) => s.evaluate()?,
            ExpandedObject::Function(f) => Object::from(f),
            ExpandedObject::Cons(c) => c.evaluate()?,
            ExpandedObject::Namespace(n) => Object::from(n),
            ExpandedObject::HeapObject(h) => (**h).evaluate()?,
        })
    }
    /// Self-evaluating types error; `Reference`s are returned,
    /// `HeapObject`s are dereferenced and `eval_to_reference`d, and
    /// `Symbol`s are looked up. `Cons`es are evaluated the same way,
    /// but they recurse on `eval_to_reference` instead of `evaluate`.
    fn eval_to_reference(&self) -> Result<Reference, EvaluatorError> {
        match *self {
            ExpandedObject::Float(_)
            | ExpandedObject::Immediate(_)
            | ExpandedObject::Function(_)
            | ExpandedObject::Namespace(_) => Err(EvaluatorError::CannotBeReferenced),
            ExpandedObject::Reference(r) => Ok(r),
            ExpandedObject::Symbol(s) => s.eval_to_reference(),
            ExpandedObject::Cons(c) => c.eval_to_reference(),
            ExpandedObject::HeapObject(h) => (**h).eval_to_reference(),
        }
    }
}
