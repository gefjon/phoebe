use allocate::Allocate;
use std::{cmp, convert, fmt, iter, mem};
use types::cons::Cons;
use types::conversions::*;
use types::{pointer_tagging, symbol, Object};

lazy_static! {
    static ref LIST_TYPE_NAME: symbol::SymRef = { ::symbol_lookup::make_symbol(b"list") };
}

#[derive(Copy, Clone)]
pub enum List {
    Nil,
    Cons(*mut Cons),
}

impl cmp::PartialEq for List {
    fn eq(&self, other: &List) -> bool {
        let mut first = *self;
        let mut second = *other;
        for (lhs, rhs) in first.zip(&mut second) {
            if !lhs.equal(rhs) {
                return false;
            }
        }
        first.next().is_none() && second.next().is_none()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(c) = <&Cons>::maybe_from(*self) {
            write!(f, "{}", c)
        } else {
            write!(f, "()")
        }
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(c) = <&Cons>::maybe_from(*self) {
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
        debug!("reversed {} into {}", self, new_list);
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
                    let &mut Cons { ref mut cdr, .. } = &mut *c;
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
            head = Cons::allocate(Cons::new(el, head));
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
                let &Cons { car, cdr, .. } = unsafe { &*c };
                let new: List = if cdr.nilp() {
                    List::Nil
                } else {
                    List::Cons(unsafe { <*mut Cons>::from_unchecked(cdr) })
                };
                *self = new;
                Some(car)
            }
        }
    }
}

impl MaybeFrom<Object> for List {
    fn maybe_from(obj: Object) -> Option<List> {
        if obj.nilp() {
            Some(List::Nil)
        } else {
            let mut cur = obj;
            loop {
                if let Some(&Cons { cdr, .. }) = <&Cons>::maybe_from(cur) {
                    cur = cdr;
                } else if cur.nilp() {
                    break;
                } else {
                    return None;
                }
            }
            Some(List::Cons(unsafe { <*mut Cons>::from_unchecked(obj) }))
        }
    }
}

impl FromUnchecked<Object> for List {
    unsafe fn from_unchecked(obj: Object) -> List {
        if obj.nilp() {
            List::Nil
        } else {
            List::Cons(<*mut Cons>::from_unchecked(obj))
        }
    }
}

impl FromObject for List {
    type Tag = pointer_tagging::ObjectTag;
    fn associated_tag() -> pointer_tagging::ObjectTag {
        pointer_tagging::ObjectTag::Cons
    }
    fn type_name() -> symbol::SymRef {
        *LIST_TYPE_NAME
    }
}

impl MaybeFrom<List> for *mut Cons {
    fn maybe_from(l: List) -> Option<*mut Cons> {
        if let List::Cons(c) = l {
            Some(c)
        } else {
            None
        }
    }
}

impl<'any> MaybeFrom<List> for &'any mut Cons {
    fn maybe_from(l: List) -> Option<&'any mut Cons> {
        <*mut Cons>::maybe_from(l).map(|r| unsafe { &mut *r })
    }
}

impl MaybeFrom<List> for *const Cons {
    fn maybe_from(l: List) -> Option<*const Cons> {
        if let List::Cons(c) = l {
            Some(c as *const Cons)
        } else {
            None
        }
    }
}

impl<'any> MaybeFrom<List> for &'any Cons {
    fn maybe_from(l: List) -> Option<&'any Cons> {
        <*const Cons>::maybe_from(l).map(|r| unsafe { &*r })
    }
}
