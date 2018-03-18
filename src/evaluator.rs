use types::{Object, ExpandedObject};
use symbol_lookup::{lookup_symbol};
use gc::gc_maybe_pass;

#[derive(Fail, Debug)]
pub enum EvaluatorError {
    #[fail(display = "I have not yet implemented EvaluatorError")]
    DummyError,
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
            
