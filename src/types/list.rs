use prelude::*;
use std::{cmp, convert, fmt, iter, mem};
use types::pointer_tagging;

lazy_static! {
    static ref LIST_TYPE_NAME: GcRef<Symbol> = { symbol_lookup::make_symbol(b"list") };
}

#[derive(Copy, Clone)]
pub enum List {
    Nil,
    Cons(GcRef<Cons>),
}

impl cmp::PartialEq for List {
    fn eq(&self, other: &List) -> bool {
        let mut first = *self;
        let mut second = *other;
        for (lhs, rhs) in (&mut first).zip(&mut second) {
            if !lhs.equal(rhs) {
                return false;
            }
        }
        first.next().is_none() && second.next().is_none()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(c) = <GcRef<Cons>>::maybe_from(*self) {
            write!(f, "{}", c)
        } else {
            write!(f, "()")
        }
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(c) = <GcRef<Cons>>::maybe_from(*self) {
            write!(f, "{:?}", c)
        } else {
            write!(f, "()")
        }
    }
}

impl List {
    pub fn nil() -> List {
        List::Nil
    }
    pub fn push(self, obj: Object) -> List {
        let c = Cons::allocate(Cons::new(obj, self.into()));
        unsafe { c.into_unchecked() }
    }
    pub fn reverse(self) -> List {
        let mut new_list = List::Nil;
        for el in self {
            new_list = new_list.push(el);
        }
        new_list
    }
    pub unsafe fn nreverse(mut self) -> List {
        let mut prev = Object::nil();
        loop {
            match self {
                List::Nil => {
                    return List::from_unchecked(prev);
                }
                List::Cons(c) => {
                    let mut copy = c;
                    let &mut Cons { ref mut cdr, .. } = copy.as_mut();
                    let next = mem::replace(cdr, prev);
                    prev = Object::from(c);
                    self = List::from_unchecked(next);
                }
            }
        }
    }
    pub fn backwards_list_from<I>(iter: I) -> List
    where
        I: iter::IntoIterator<Item = Object>,
    {
        let mut head = Object::nil();
        for el in iter {
            head = Object::from(Cons::allocate(Cons::new(el, head)));
        }
        unsafe { List::from_unchecked(head) }
    }
}

impl<O> iter::FromIterator<O> for List
where
    Object: convert::From<O>,
{
    fn from_iter<T: iter::IntoIterator<Item = O>>(iter: T) -> List {
        let backwards = List::backwards_list_from(iter.into_iter().map(Object::from));
        unsafe { backwards.nreverse() }
    }
}

impl convert::From<List> for Object {
    fn from(l: List) -> Object {
        match l {
            List::Nil => Object::nil(),
            List::Cons(c) => Object::from(c),
        }
    }
}

impl iter::Iterator for List {
    type Item = Object;
    fn next(&mut self) -> Option<Object> {
        match *self {
            List::Nil => None,
            List::Cons(c) => {
                let Cons { car, cdr, .. } = *c;
                let new: List = if cdr.nilp() {
                    List::Nil
                } else {
                    List::Cons(unsafe { cdr.into_unchecked() })
                };
                *self = new;
                Some(car)
            }
        }
    }
}

impl MaybeFrom<GcRef<Cons>> for List {
    fn maybe_from(c: GcRef<Cons>) -> Option<List> {
        let Cons { cdr, .. } = *c;

        let mut cur = cdr;
        loop {
            if let Some(c) = GcRef::<Cons>::maybe_from(cur) {
                let Cons { cdr, .. } = *c;
                cur = cdr;
            } else if cur.nilp() {
                break;
            } else {
                return None;
            }
        }
        Some(List::Cons(c))
    }
    fn try_convert_from(c: GcRef<Cons>) -> Result<List, ConversionError> {
        if let Some(l) = List::maybe_from(c) {
            Ok(l)
        } else {
            Err(ConversionError::wanted(List::type_name()))
        }
    }
}

impl FromUnchecked<GcRef<Cons>> for List {
    unsafe fn from_unchecked(c: GcRef<Cons>) -> List {
        List::Cons(c)
    }
}

impl MaybeFrom<Object> for List {
    fn maybe_from(obj: Object) -> Option<List> {
        if obj.nilp() {
            Some(List::Nil)
        } else {
            let mut cur = obj;
            loop {
                if let Some(r) = GcRef::<Cons>::maybe_from(cur) {
                    let Cons { cdr, .. } = *r;
                    cur = cdr;
                } else if cur.nilp() {
                    break;
                } else {
                    return None;
                }
            }
            Some(List::Cons(unsafe { GcRef::from_unchecked(obj) }))
        }
    }
    fn try_convert_from(obj: Object) -> Result<List, ConversionError> {
        if let Some(t) = List::maybe_from(obj) {
            Ok(t)
        } else {
            Err(ConversionError::wanted(List::type_name()))
        }
    }
}

impl FromUnchecked<Object> for List {
    unsafe fn from_unchecked(obj: Object) -> List {
        if obj.nilp() {
            List::Nil
        } else {
            List::Cons(GcRef::from_unchecked(obj))
        }
    }
}

impl FromObject for List {
    type Tag = pointer_tagging::ObjectTag;
    fn associated_tag() -> pointer_tagging::ObjectTag {
        pointer_tagging::ObjectTag::Cons
    }
    fn type_name() -> GcRef<Symbol> {
        *LIST_TYPE_NAME
    }
}

impl MaybeFrom<List> for GcRef<Cons> {
    fn maybe_from(l: List) -> Option<GcRef<Cons>> {
        if let List::Cons(c) = l {
            Some(c)
        } else {
            None
        }
    }
    fn try_convert_from(obj: List) -> Result<GcRef<Cons>, ConversionError> {
        if let Some(t) = <GcRef<Cons>>::maybe_from(obj) {
            Ok(t)
        } else {
            Err(ConversionError::wanted(<GcRef<Cons>>::type_name()))
        }
    }
}
