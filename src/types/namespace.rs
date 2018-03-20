use super::pointer_tagging::{ObjectTag, PointerTag};
use std::collections::HashMap;
use super::{Object, heap_object, reference};
use types::symbol::SymRef;
use super::conversions::*;
use gc::{GcMark, GarbageCollected};
use std::{convert, fmt, iter};
use std::default::Default;

lazy_static! {
    static ref NAMESPACE_TYPE_NAME: SymRef = {
        ::symbol_lookup::make_symbol(b"namespace")
    };
}

#[derive(Clone, Debug)]
pub enum Namespace {
    Heap {
        gc_marking: GcMark,
        name: Option<Object>,
        table: HashMap<SymRef, *mut heap_object::HeapObject>,
    },
    Stack {
        gc_marking: GcMark,
        table: HashMap<SymRef, reference::Reference>,
    }
}

impl iter::FromIterator<(SymRef, reference::Reference)> for Namespace {
    fn from_iter<I>(iter: I) -> Namespace
    where I: iter::IntoIterator<Item = (SymRef, reference::Reference)> {
        let table = iter.into_iter().collect();
        Namespace::Stack {
            gc_marking: GcMark::default(),
            table,
        }
    }
}

impl Default for Namespace {
    fn default() -> Namespace {
        Namespace::Heap {
            gc_marking: GcMark::default(),
            name: None,
            table: HashMap::new(),
        }
    }
}

impl Namespace {
    pub fn with_name(mut self, n: Object) -> Namespace {
        match self {
            Namespace::Heap { ref mut name, .. } => {
                *name = Some(n);
            }
            Namespace::Stack { .. } => {
                panic!("Attempt to name a stack Namespace");
            }
        }
        self
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Namespace::Heap { name: Some(name), .. } => write!(f, "[namespace {}]", name),
            Namespace::Heap { name: None, .. } => write!(f, "[namespace ANONYMOUS]"),
            Namespace::Stack { .. } => write!(f, "[namespace STACK-FRAME]"),
        }
    }
}

impl GarbageCollected for Namespace {
    fn my_marking(&self) -> &GcMark {
        match *self {
            Namespace::Heap { ref gc_marking, .. } => gc_marking,
            Namespace::Stack { ref gc_marking, .. } => gc_marking,
        }
    }
    fn my_marking_mut(&mut self) -> &mut GcMark {
        match *self {
            Namespace::Heap { ref mut gc_marking, .. } => gc_marking,
            Namespace::Stack { ref mut gc_marking, .. } => gc_marking,
        }
    }
    fn gc_mark_children(&mut self, mark: GcMark) {
        match *self {
            Namespace::Heap { ref mut table, .. } => {
                for (&sym, &mut heapobj) in table {
                    sym.gc_mark(mark);
                    unsafe { &mut *heapobj }.gc_mark(mark);
                }
            }
            Namespace::Stack { ref mut table, .. } => {
                for (&sym, &mut reference) in table {
                    sym.gc_mark(mark);
                    (*reference).gc_mark(mark);
                }
            }
        }
    }
}

impl convert::From<*mut Namespace> for Object {
    fn from(n: *mut Namespace) -> Object {
        Object(ObjectTag::Namespace.tag(n as u64))
    }
}

impl FromUnchecked<Object> for *mut Namespace {
    unsafe fn from_unchecked(obj: Object) -> *mut Namespace {
        debug_assert!(<*mut Namespace>::is_type(obj));
        <*mut Namespace>::associated_tag().untag(obj.0) as *mut Namespace
    }
}

impl FromObject for *mut Namespace {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Namespace
    }
    fn type_name() -> SymRef {
        *NAMESPACE_TYPE_NAME
    }
}

unsafe impl Send for Namespace {}
unsafe impl Sync for Namespace {}
