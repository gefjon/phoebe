//! This module contains the framework for evaluation, which manifests
//! as:
//!
//! * the trait `Evaluate`
//!
//! * the thread-local state function `should_eval_to_reference`
//!
//! * the function `eval_to_reference`, which is called by `setf`
//!
//! * the unsafe function `eval_from_stack`, which is a part of the
//!   read-eval-print loop

use crate::prelude::*;
use crate::types::ExpandedObject;
use std::cell::Cell;

thread_local! {
    static EVAL_TO_REFERENCE: Cell<bool> = {
        Cell::new(false)
    };
}

fn should_eval_to_reference() -> bool {
    EVAL_TO_REFERENCE.with(|r| r.get())
}

pub fn eval_to_reference(obj: Object) -> Object {
    let old_e = EVAL_TO_REFERENCE.with(|r| r.replace(true));
    let res = obj.evaluate();
    EVAL_TO_REFERENCE.with(|r| {
        r.set(old_e);
    });
    res
}

pub unsafe fn eval_from_stack() {
    let to_eval = match stack::nth_arg(0) {
        Ok(o) => *o,
        Err(e) => {
            stack::close_stack_frame_and_return(<GcRef<Error>>::from(e).into());
            return;
        }
    };
    stack::close_stack_frame_and_return(to_eval.evaluate());
}

pub trait Evaluate {
    fn evaluate(&self) -> Object;
}

impl Evaluate for Object {
    /// `evaluate`, like most operations on `Object`s, involves
    /// deconstructing `self` into an `ExpandedObject` and then
    /// calling `evaluate` on that.
    fn evaluate(&self) -> Object {
        info!("Evaluating {}.", self);

        (*self)?;

        let mut o = self.expand_quiet().evaluate();

        if !should_eval_to_reference() {
            while let Some(r) = Reference::maybe_from(o) {
                o = *r;
            }
        }

        o
    }
}

impl Evaluate for ExpandedObject {
    /// Floats, `Immediate`s, `Function`s and `Namespace`s are all
    /// self-evaluating. `Reference`s evaluate to the value they
    /// dereference to. `HeapObject`s evaluate by dereferencing and
    /// evaluating themselves. `Symbol`s are looked up. `Cons`es are
    /// the only `Object`s with a serious, beefy `evaluate`
    /// implementation.
    fn evaluate(&self) -> Object {
        match *self {
            ExpandedObject::Float(n) => Object::from(n),
            ExpandedObject::Immediate(i) => Object::from(i),
            ExpandedObject::Reference(ref r) => **r,
            ExpandedObject::Symbol(s) => s.evaluate(),
            ExpandedObject::Function(f) => Object::from(f),
            ExpandedObject::Cons(c) => c.evaluate(),
            ExpandedObject::Namespace(n) => Object::from(n),
            ExpandedObject::HeapObject(h) => (**h).evaluate(),
            ExpandedObject::QuietError(e) => Object::quiet_error(e),
        }
    }
}
