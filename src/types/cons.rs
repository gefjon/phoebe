use types::{reference, symbol, Object};
use types::conversions::*;
use gc::{GarbageCollected, GcMark};
use types::pointer_tagging::{ObjectTag, PointerTag};
use std::{cmp, convert, fmt};
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

impl cmp::PartialEq for Cons {
    fn eq(&self, other: &Cons) -> bool {
        self.car.equal(other.car) && self.cdr.equal(other.cdr)
    }
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
        use types::function::Function;
        use types::list::List;

        let mut l: List = Object::from(self as *const Cons as *mut Cons).try_into()?;
        let f = l.next().unwrap();
        let func: &Function = f.evaluate()?.try_into()?;
        func.call(l)
    }
    fn eval_to_reference(&self) -> Result<reference::Reference, EvaluatorError> {
        use types::function::Function;
        use types::list::List;

        let mut l: List = Object::from(self as *const Cons as *mut Cons).try_into()?;
        let f = l.next().unwrap();
        let func: &Function = f.evaluate()?.try_into()?;
        func.call_to_reference(l)
    }
}

impl fmt::Display for Cons {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Cons {
            car, cdr: mut curr, ..
        } = self;
        write!(f, "({}", car)?;
        loop {
            if let Some(&Cons { car, cdr, .. }) = <&Cons>::maybe_from(curr) {
                curr = cdr;
                write!(f, " {}", car)?;
            } else if curr.nilp() {
                break;
            } else {
                write!(f, " . {}", curr)?;
                break;
            }
        }
        write!(f, ")")
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

#[cfg(test)]
mod test {
    use super::*;
    use types::Object;
    use allocate::Allocate;
    #[test]
    fn format_a_cons() {
        let c = Cons::new(
            Object::from(1i32),
            Cons::allocate(Cons::new(
                Object::from(2i32),
                Cons::allocate(Cons::new(Object::from(3i32), Object::from(4i32))),
            )),
        );
        assert_eq!(format!("{}", c), "(1 2 3 . 4)");
    }
}
