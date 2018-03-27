use super::conversions::*;
use super::pointer_tagging::{ObjectTag, PointerTag};
use super::{heap_object, reference, symbol, Object};
use gc::{GarbageCollected, GcMark};
use std::collections::HashMap;
use std::default::Default;
use std::{convert, fmt, iter, ops};
use symbol_lookup;
use types::symbol::SymRef;

lazy_static! {
    static ref NAMESPACE_TYPE_NAME: SymRef = { symbol_lookup::make_symbol(b"namespace") };
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct NamespaceRef(*mut Namespace);

impl NamespaceRef {
    pub fn gc_mark(self, m: GcMark) {
        unsafe { &mut *(self.0) }.gc_mark(m);
    }

    pub fn clone_if_needed(self) -> NamespaceRef {
        use allocate::Allocate;

        let obj = Namespace::allocate(match *self {
            Namespace::Heap {
                parent: Some(p),
                ref table,
                name,
                ..
            } => Namespace::Heap {
                parent: Some(p.clone_if_needed()),
                table: table.clone(),
                name,
                gc_marking: GcMark::default(),
            },
            Namespace::Heap { parent: None, .. } => {
                return self;
            }
            Namespace::Stack {
                ref table, parent, ..
            } => {
                let table = table
                    .iter()
                    .map(|(&s, &r)| {
                        use allocate::Allocate;
                        use types::heap_object::HeapObject;

                        let h = HeapObject::allocate(HeapObject::around(*r));
                        (s, unsafe { <*mut HeapObject>::from_unchecked(h) })
                    })
                    .collect();
                let parent = parent.and_then(|p| Some(p.clone_if_needed()));
                Namespace::Heap {
                    gc_marking: GcMark::default(),
                    name: None,
                    table,
                    parent,
                }
            }
        });
        unsafe { obj.into_unchecked() }
    }
}

impl fmt::Display for NamespaceRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", **self)
    }
}

impl fmt::Debug for NamespaceRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[nref 0x{:p} -> {:?}]", self.0, **self)
    }
}

impl AsRef<Namespace> for NamespaceRef {
    fn as_ref(&self) -> &Namespace {
        unsafe { &*(self.0) }
    }
}

impl AsMut<Namespace> for NamespaceRef {
    fn as_mut(&mut self) -> &mut Namespace {
        unsafe { &mut *(self.0) }
    }
}

impl ops::Deref for NamespaceRef {
    type Target = Namespace;
    fn deref(&self) -> &Namespace {
        self.as_ref()
    }
}

impl ops::DerefMut for NamespaceRef {
    fn deref_mut(&mut self) -> &mut Namespace {
        self.as_mut()
    }
}

impl convert::From<NamespaceRef> for *mut Namespace {
    fn from(n: NamespaceRef) -> *mut Namespace {
        n.0
    }
}

impl<'any> convert::From<&'any mut Namespace> for NamespaceRef {
    fn from(n: &mut Namespace) -> NamespaceRef {
        NamespaceRef(n as *mut Namespace)
    }
}

impl convert::From<*mut Namespace> for NamespaceRef {
    fn from(n: *mut Namespace) -> NamespaceRef {
        NamespaceRef(n)
    }
}

impl convert::From<NamespaceRef> for Object {
    fn from(n: NamespaceRef) -> Object {
        Object::from(<*mut Namespace>::from(n))
    }
}

impl FromUnchecked<Object> for NamespaceRef {
    unsafe fn from_unchecked(obj: Object) -> NamespaceRef {
        NamespaceRef(<*mut Namespace>::from_unchecked(obj))
    }
}

impl FromObject for NamespaceRef {
    type Tag = <*mut Namespace as FromObject>::Tag;
    fn associated_tag() -> Self::Tag {
        <*mut Namespace as FromObject>::associated_tag()
    }
    fn type_name() -> symbol::SymRef {
        <*mut Namespace as FromObject>::type_name()
    }
}

unsafe impl Send for NamespaceRef {}
unsafe impl Sync for NamespaceRef {}

#[derive(Debug, Clone)]
pub enum Namespace {
    Heap {
        gc_marking: GcMark,
        name: Option<Object>,
        table: HashMap<SymRef, *mut heap_object::HeapObject>,
        parent: Option<NamespaceRef>,
    },
    Stack {
        gc_marking: GcMark,
        table: HashMap<SymRef, reference::Reference>,
        parent: Option<NamespaceRef>,
    },
}

impl iter::FromIterator<(SymRef, reference::Reference)> for Namespace {
    fn from_iter<I>(iter: I) -> Namespace
    where
        I: iter::IntoIterator<Item = (SymRef, reference::Reference)>,
    {
        let table = iter.into_iter().collect();
        Namespace::Stack {
            gc_marking: GcMark::default(),
            table,
            parent: None,
        }
    }
}

impl iter::FromIterator<(SymRef, Object)> for Namespace {
    fn from_iter<I>(iter: I) -> Namespace
    where
        I: iter::IntoIterator<Item = (SymRef, Object)>,
    {
        use allocate::Allocate;
        use types::heap_object::HeapObject;

        let table = iter.into_iter()
            .map(|(r, o)| {
                let h = HeapObject::allocate(HeapObject::around(o));
                (r, unsafe { <*mut HeapObject>::from_unchecked(h) })
            })
            .collect();
        Namespace::Heap {
            gc_marking: GcMark::default(),
            name: None,
            table,
            parent: None,
        }
    }
}

impl Default for Namespace {
    fn default() -> Namespace {
        Namespace::Heap {
            gc_marking: GcMark::default(),
            name: None,
            table: HashMap::new(),
            parent: None,
        }
    }
}

impl Namespace {
    /// This function builds and allocates an env to be used by `let`,
    /// though it *does not* push it to the `ENV_STACK`.
    pub fn create_let_env(pairs: &[(SymRef, Object)]) -> NamespaceRef {
        use allocate::Allocate;

        let nmspc: Namespace = pairs.iter().cloned().collect();

        let nmspc = Namespace::allocate(nmspc.with_parent(symbol_lookup::current_env()));
        unsafe { nmspc.into_unchecked() }
    }

    /// This function builds and allocates a function's running
    /// environment, though it *does not* push it to the `ENV_STACK`.
    pub fn create_stack_env(
        pairs: &[(SymRef, reference::Reference)],
        parent: NamespaceRef,
    ) -> NamespaceRef {
        use allocate::Allocate;

        let nmspc: Namespace = pairs.iter().cloned().collect();

        let nmspc = Namespace::allocate(nmspc.with_parent(parent));
        unsafe { nmspc.into_unchecked() }
    }

    pub fn parent(&self) -> Option<NamespaceRef> {
        match *self {
            Namespace::Stack { parent, .. } | Namespace::Heap { parent, .. } => parent,
        }
    }
    pub fn with_parent(self, parent: NamespaceRef) -> Namespace {
        match self {
            Namespace::Stack { table, .. } => Namespace::Stack {
                gc_marking: GcMark::default(),
                table,
                parent: Some(parent),
            },
            Namespace::Heap { name, table, .. } => Namespace::Heap {
                gc_marking: GcMark::default(),
                name,
                table,
                parent: Some(parent),
            },
        }
    }
    pub fn needs_clone(&self) -> bool {
        if let Namespace::Stack { .. } = *self {
            true
        } else if let Some(n) = self.parent() {
            n.needs_clone()
        } else {
            false
        }
    }
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
    pub fn get_sym_ref(&self, sym: SymRef) -> Option<reference::Reference> {
        match *self {
            Namespace::Heap { ref table, .. } => table
                .get(&sym)
                .map(|&h| reference::Reference::from(unsafe { &mut *h }))
                .or_else(|| self.parent().and_then(|n| n.get_sym_ref(sym))),
            Namespace::Stack { ref table, .. } => table
                .get(&sym)
                .cloned()
                .or_else(|| self.parent().and_then(|n| n.get_sym_ref(sym))),
        }
    }

    /// This function may have unwanted behavior: it *will not* search
    /// parent envs. It is called by
    /// `symbol_lookup::make_from_[default_]global_namespace`.
    pub fn make_sym_ref(&mut self, sym: SymRef) -> reference::Reference {
        use allocate::Allocate;
        use std::default::Default;

        match *self {
            Namespace::Heap { ref mut table, .. } => {
                let p = *(table.entry(sym).or_insert_with(|| {
                    let h = heap_object::HeapObject::allocate(heap_object::HeapObject::around(
                        Object::default(),
                    ));
                    unsafe { h.into_unchecked() }
                }));
                unsafe { &mut *p }.into()
            }
            Namespace::Stack { .. } => panic!("Attempt to insert into a stack namespace"),
        }
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Namespace::Heap {
                name: Some(name), ..
            } => write!(f, "[namespace {}]", name),
            Namespace::Heap { name: None, .. } => write!(f, "[namespace ANONYMOUS]"),
            Namespace::Stack { .. } => write!(f, "[namespace STACK-FRAME]"),
        }
    }
}

impl GarbageCollected for Namespace {
    fn my_marking(&self) -> &GcMark {
        match *self {
            Namespace::Heap { ref gc_marking, .. } | Namespace::Stack { ref gc_marking, .. } => {
                gc_marking
            }
        }
    }
    fn my_marking_mut(&mut self) -> &mut GcMark {
        match *self {
            Namespace::Heap {
                ref mut gc_marking, ..
            }
            | Namespace::Stack {
                ref mut gc_marking, ..
            } => gc_marking,
        }
    }
    fn gc_mark_children(&mut self, mark: GcMark) {
        match *self {
            Namespace::Heap { ref mut table, .. } => for (&sym, &mut heapobj) in table {
                sym.gc_mark(mark);
                unsafe { &mut *heapobj }.gc_mark(mark);
            },
            Namespace::Stack { ref mut table, .. } => for (&sym, &mut reference) in table {
                sym.gc_mark(mark);
                (*reference).gc_mark(mark);
            },
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
