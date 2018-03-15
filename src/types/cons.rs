use types::{Object, reference, symbol};
use types::conversions::*;
use gc::{GcMark, GarbageCollected};
use types::pointer_tagging::{ObjectTag, PointerTag};
use std::{convert, fmt};
use evaluator::{Evaluate, EvaluatorError};

lazy_static! {
    static ref CONS_TYPE_NAME: symbol::SymRef = {
        ::symbol_lookup::make_symbol(b"cons")
    };
}

#[derive(Clone, Debug)]
pub struct Cons {
    gc_marking: GcMark,
    pub car: Object,
    pub cdr: Object,
}

impl Cons {
    pub fn new(car: Object, cdr: Object) -> Cons {
        Cons {
            gc_marking: GcMark::default(),
            car,
            cdr,
        }
    }
    pub fn ref_car(&mut self) -> reference::Reference {
        reference::Reference::from(&mut self.car)
    }
    pub fn ref_cdr(&mut self) -> reference::Reference {
        reference::Reference::from(&mut self.cdr)
    }
}

impl Evaluate for Cons {
    fn evaluate(&self) -> Result<Object, EvaluatorError> {
        unimplemented!()
    }
}

impl fmt::Display for Cons {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

impl convert::From<*mut Cons> for Object {
    fn from(c: *mut Cons) -> Object {
        Object(ObjectTag::Cons.tag(c as u64))
    }
}

impl FromUnchecked<Object> for *mut Cons {
    unsafe fn from_unchecked(obj: Object) -> *mut Cons {
        debug_assert!(<*mut Cons>::is_type(obj));
        <*mut Cons>::associated_tag().untag(obj.0) as *mut Cons
    }
}

impl FromObject for *mut Cons {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Cons
    }
    fn type_name() -> symbol::SymRef {
        *CONS_TYPE_NAME
    }
}

impl GarbageCollected for Cons {
    fn my_marking(&self) -> &GcMark {
        &self.gc_marking
    }
    fn my_marking_mut(&mut self) -> &mut GcMark {
        &mut self.gc_marking
    }
    fn gc_mark_children(&mut self, mark: GcMark) {
        self.car.gc_mark(mark);
        self.cdr.gc_mark(mark);
    }
}
