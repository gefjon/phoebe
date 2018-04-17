use super::pointer_tagging::{ObjectTag, PointerTag};
use prelude::*;
use std::collections::HashMap;
use std::default::Default;
use std::sync::RwLock;
use std::{convert, fmt, iter};

lazy_static! {
    static ref NAMESPACE_TYPE_NAME: GcRef<Symbol> = { symbol_lookup::make_symbol(b"namespace") };
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct NamespaceRef(*mut Namespace);

impl GcRef<Namespace> {
    pub fn clone_if_needed(self) -> GcRef<Namespace> {
        Namespace::allocate(match *self {
            Namespace::Heap {
                parent: Some(p),
                ref table,
                name,
                ..
            } => Namespace::Heap {
                parent: Some(p.clone_if_needed()),
                table: RwLock::new((*(table.read().unwrap())).clone()),
                name,
                gc_marking: GcMark::default(),
            },
            Namespace::Heap { parent: None, .. } => {
                return self;
            }
            Namespace::Stack {
                ref table, parent, ..
            } => {
                let table = RwLock::new(
                    table
                        .read()
                        .unwrap()
                        .iter()
                        .map(|(&s, r)| (s, HeapObject::allocate(HeapObject::around(**r))))
                        .collect(),
                );
                let parent = parent.and_then(|p| Some(p.clone_if_needed()));
                Namespace::Heap {
                    gc_marking: GcMark::default(),
                    name: None,
                    table,
                    parent,
                }
            }
        })
    }
}

impl convert::From<GcRef<Namespace>> for Object {
    fn from(n: GcRef<Namespace>) -> Object {
        Object::from_raw(<GcRef<Namespace>>::associated_tag().tag(n.into_ptr() as u64))
    }
}

#[derive(Debug)]
pub enum Namespace {
    Heap {
        gc_marking: GcMark,
        name: Option<Object>,
        table: RwLock<HashMap<GcRef<Symbol>, GcRef<HeapObject>>>,
        parent: Option<GcRef<Namespace>>,
    },
    Stack {
        gc_marking: GcMark,
        table: RwLock<HashMap<GcRef<Symbol>, Reference>>,
        parent: Option<GcRef<Namespace>>,
    },
}

impl Clone for Namespace {
    fn clone(&self) -> Namespace {
        match *self {
            Namespace::Heap {
                name,
                ref table,
                parent,
                ..
            } => Namespace::Heap {
                name,
                table: RwLock::new((*(table.read().unwrap())).clone()),
                parent,
                gc_marking: GcMark::default(),
            },
            Namespace::Stack {
                ref table, parent, ..
            } => Namespace::Stack {
                table: RwLock::new((*(table.read().unwrap())).clone()),
                parent,
                gc_marking: GcMark::default(),
            },
        }
    }
}

impl iter::FromIterator<(GcRef<Symbol>, Reference)> for Namespace {
    fn from_iter<I>(iter: I) -> Namespace
    where
        I: iter::IntoIterator<Item = (GcRef<Symbol>, Reference)>,
    {
        let table = RwLock::new(iter.into_iter().collect());
        Namespace::Stack {
            gc_marking: GcMark::default(),
            table,
            parent: None,
        }
    }
}

impl iter::FromIterator<(GcRef<Symbol>, Object)> for Namespace {
    fn from_iter<I>(iter: I) -> Namespace
    where
        I: iter::IntoIterator<Item = (GcRef<Symbol>, Object)>,
    {
        let table = RwLock::new(
            iter.into_iter()
                .map(|(r, o)| {
                    let h = HeapObject::allocate(HeapObject::around(o));
                    (r, h)
                })
                .collect(),
        );
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
            table: RwLock::new(HashMap::new()),
            parent: None,
        }
    }
}

impl Namespace {
    pub fn lowest_parent<'any>(mut me: GcRef<Namespace>) -> &'any mut Option<GcRef<Namespace>> {
        loop {
            match *me {
                Namespace::Heap {
                    parent: Some(p), ..
                }
                | Namespace::Stack {
                    parent: Some(p), ..
                } => {
                    me = p;
                }
                _ => {
                    break;
                }
            }
        }

        match *me {
            Namespace::Heap { ref mut parent, .. } | Namespace::Stack { ref mut parent, .. } => {
                unsafe {
                    // Any references to garbage-collected items are
                    // valid for any lifetime, including `'static`, as
                    // long as the references are correctly included
                    // in the mark-and-sweep system.
                    &mut *(parent as *mut Option<GcRef<Namespace>>)
                }
            }
        }
    }
    /// This function builds and allocates an env to be used by `let`,
    /// though it *does not* push it to the `ENV_STACK`.
    pub fn create_let_env(pairs: &[(GcRef<Symbol>, Object)]) -> GcRef<Namespace> {
        let nmspc: Namespace = pairs.iter().cloned().collect();

        Namespace::allocate(nmspc.with_parent(symbol_lookup::current_env()))
    }

    /// This function builds and allocates a function's running
    /// environment, though it *does not* push it to the `ENV_STACK`.
    pub fn create_stack_env(
        pairs: &[(GcRef<Symbol>, Reference)],
        parent: GcRef<Namespace>,
    ) -> GcRef<Namespace> {
        let nmspc: Namespace = pairs.iter().cloned().collect();

        Namespace::allocate(nmspc.with_parent(parent))
    }

    pub fn parent(&self) -> Option<GcRef<Namespace>> {
        match *self {
            Namespace::Stack { parent, .. } | Namespace::Heap { parent, .. } => parent,
        }
    }
    pub fn with_parent(self, parent: GcRef<Namespace>) -> Namespace {
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
    pub fn with_maybe_name(mut self, n: Option<Object>) -> Namespace {
        match self {
            Namespace::Heap { ref mut name, .. } => {
                *name = n;
            }
            Namespace::Stack { .. } => {
                panic!("Attempt to name a stack Namespace");
            }
        }
        self
    }
    pub fn make_sym_ref_search_parent(&mut self, sym: GcRef<Symbol>) -> Reference {
        self.get_sym_ref(sym)
            .unwrap_or_else(|| self.make_sym_ref(sym))
    }
    pub fn get_sym_ref(&self, sym: GcRef<Symbol>) -> Option<Reference> {
        match *self {
            Namespace::Heap { ref table, .. } => table
                .read()
                .unwrap()
                .get(&sym)
                .map(|&h| Reference::from(h))
                .or_else(|| self.parent().and_then(|n| n.get_sym_ref(sym))),
            Namespace::Stack { ref table, .. } => table
                .read()
                .unwrap()
                .get(&sym)
                .cloned()
                .or_else(|| self.parent().and_then(|n| n.get_sym_ref(sym))),
        }
    }

    /// This function may have unwanted behavior: it *will not* search
    /// parent envs. It is called by
    /// `symbol_lookup::make_from_[default_]global_namespace`.
    pub fn make_sym_ref(&mut self, sym: GcRef<Symbol>) -> Reference {
        use std::default::Default;

        match *self {
            Namespace::Heap { ref mut table, .. } => {
                let p = *(table.write().unwrap().entry(sym).or_insert_with(|| {
                    HeapObject::allocate(HeapObject::around(Object::default()))
                }));
                p.into()
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
    type ConvertFrom = Namespace;
    fn alloc_one_and_initialize(n: Namespace) -> ::std::ptr::NonNull<Namespace> {
        use std::alloc::{Alloc, Global};
        use std::ptr;
        let nn = Global
            .alloc_one::<Namespace>()
            .unwrap_or_else(|_| Global.oom());
        let p = nn.as_ptr();
        unsafe { ptr::write(p, n) };
        nn
    }
    fn my_marking(&self) -> &GcMark {
        match *self {
            Namespace::Heap { ref gc_marking, .. } | Namespace::Stack { ref gc_marking, .. } => {
                gc_marking
            }
        }
    }
    fn gc_mark_children(&mut self, mark: usize) {
        match *self {
            Namespace::Heap {
                ref mut table,
                parent,
                ..
            } => for (sym, heapobj) in table.read().unwrap().iter() {
                sym.clone().gc_mark(mark);
                heapobj.clone().gc_mark(mark);
                parent.map(|p| p.gc_mark(mark));
            },
            Namespace::Stack {
                ref mut table,
                parent,
                ..
            } => for (sym, reference) in table.read().unwrap().iter() {
                sym.clone().gc_mark(mark);
                (*reference).gc_mark(mark);
                parent.map(|p| p.gc_mark(mark));
            },
        }
    }
}

impl FromUnchecked<Object> for GcRef<Namespace> {
    unsafe fn from_unchecked(obj: Object) -> GcRef<Namespace> {
        debug_assert!(Self::is_type(obj));
        GcRef::from_ptr(Self::associated_tag().untag(obj.0) as *mut Namespace)
    }
}

impl FromObject for GcRef<Namespace> {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Namespace
    }
    fn type_name() -> GcRef<Symbol> {
        *NAMESPACE_TYPE_NAME
    }
}

unsafe impl Send for Namespace {}
unsafe impl Sync for Namespace {}
