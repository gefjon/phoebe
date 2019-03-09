use crate::prelude::*;
use crate::types::pointer_tagging::{ObjectTag, PointerTag};
use std::{cmp, convert, fmt};

lazy_static! {
    static ref CONS_TYPE_NAME: GcRef<Symbol> = { symbol_lookup::make_symbol(b"cons") };
}

#[derive(Debug)]
pub struct Cons {
    gc_marking: GcMark,
    pub car: Object,
    pub cdr: Object,
}

impl Clone for Cons {
    fn clone(&self) -> Cons {
        let &Cons { car, cdr, .. } = self;
        Cons {
            car,
            cdr,
            gc_marking: GcMark::default(),
        }
    }
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
    pub fn ref_car(&mut self) -> Reference {
        Reference::from(&mut self.car)
    }
    pub fn ref_cdr(&mut self) -> Reference {
        Reference::from(&mut self.cdr)
    }
}

impl Evaluate for Cons {
    fn evaluate(&self) -> Object {
        let mut l =
            List::try_convert_from(unsafe { GcRef::from_ptr(self as *const Cons as *mut Cons) })?;
        let f = l.next().unwrap();
        let func = <GcRef<Function>>::try_convert_from(f.evaluate()?)?;
        func.call(l)
    }
}

impl fmt::Display for Cons {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Cons {
            car, cdr: mut curr, ..
        } = *self;
        write!(f, "({}", car)?;
        loop {
            if curr.nilp() {
                break;
            } else if let Some(c) = <GcRef<Cons>>::maybe_from(curr) {
                let Cons { car, cdr, .. } = *c;
                curr = cdr;
                write!(f, " {}", car)?;
            } else {
                write!(f, " . {}", curr)?;
                break;
            }
        }
        write!(f, ")")
    }
}

impl convert::From<GcRef<Cons>> for Object {
    fn from(c: GcRef<Cons>) -> Object {
        Object::from_raw(ObjectTag::Cons.tag(c.into_ptr() as u64))
    }
}

impl FromUnchecked<Object> for GcRef<Cons> {
    unsafe fn from_unchecked(obj: Object) -> Self {
        debug_assert!(Self::is_type(obj));
        GcRef::from_ptr(Self::associated_tag().untag(obj.0) as *mut Cons)
    }
}

impl FromObject for GcRef<Cons> {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Cons
    }
    fn type_name() -> GcRef<Symbol> {
        *CONS_TYPE_NAME
    }
}

impl GarbageCollected for Cons {
    type ConvertFrom = Cons;
    fn alloc_one_and_initialize(o: Self) -> ::std::ptr::NonNull<Self> {
        use std::alloc::{Alloc, Global};
        use std::ptr;
        let nn = Global.alloc_one().unwrap();
        let p = nn.as_ptr();
        unsafe { ptr::write(p, o) };
        nn
    }
    fn my_marking(&self) -> &GcMark {
        &self.gc_marking
    }
    fn gc_mark_children(&mut self, mark: usize) {
        self.car.gc_mark(mark);
        self.cdr.gc_mark(mark);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::gc::GarbageCollected;
    use crate::types::Object;
    #[test]
    fn format_a_cons() {
        let c = Cons::new(
            Object::from(1i32),
            Object::from(Cons::allocate(Cons::new(
                Object::from(2i32),
                Object::from(Cons::allocate(Cons::new(
                    Object::from(3i32),
                    Object::from(4i32),
                ))),
            ))),
        );
        assert_eq!(format!("{}", c), "(1 2 3 . 4)");
    }
}
