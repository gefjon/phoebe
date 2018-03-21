use types::{namespace, reference, Object};
use types::symbol::{SymRef, Symbol};
use allocate::Allocate;
use types::conversions::*;
use std::{cell, sync};
use std::collections::HashMap;
use gc;

type Scope = Vec<&'static mut namespace::Namespace>;

static GLOBAL_NAMESPACE_NAME: &[u8] = b"global-namespace";

lazy_static! {
    pub static ref SYMBOLS_HEAP: sync::Mutex<HashMap<Vec<u8>, SymRef>> = {
        sync::Mutex::new(HashMap::new())
    };
}

thread_local! {
    static SCOPE: cell::RefCell<Scope> = {
        let global_namespace = namespace::Namespace::allocate(
            namespace::Namespace::default().with_name(
                Object::from(make_symbol(GLOBAL_NAMESPACE_NAME))
            )
        );
        cell::RefCell::new(vec![unsafe { global_namespace.into_unchecked() }])
    };
}

pub fn add_to_global(name: SymRef, obj: Object) {
    SCOPE.with(|s| {
        *(s.borrow_mut()[0].make_sym_ref(name)) = obj;
    });
}

/// BUG: The `SCOPE` is thread local, but garbage collection is done
/// globally. This means that the garbage collector cannot mark other
/// threads' scopes and may deallocate them prematurely.
pub fn gc_mark_scope(m: gc::GcMark) {
    use gc::GarbageCollected;
    SCOPE.with(|s| {
        for nmspc in s.borrow_mut().iter_mut() {
            nmspc.gc_mark(m);
        }
    });
}

pub fn add_heap_scope(n: &[(SymRef, Object)]) {
    let nmspc = namespace::Namespace::allocate(n.iter().cloned().collect());
    SCOPE.with(|s| s.borrow_mut().push(unsafe { nmspc.into_unchecked() }));
}

pub fn add_namespace_to_scope(n: &[(SymRef, reference::Reference)]) {
    let nmspc = namespace::Namespace::allocate(n.iter().cloned().collect());
    SCOPE.with(|s| {
        s.borrow_mut().push(unsafe { nmspc.into_unchecked() });
    });
}

/// This call closes the namespace created by `add_namespace_to_scope`
/// or `add_heap_scope`. It is intended to be called only for local
/// bindings such as `let` and function calls, and should only ever be
/// called after a corresponding `add_namespace_to_scope` when it is
/// known that the correct namespace is the top of the scope. Scopes
/// should also be destroyed *before* their stack frames are closed
/// using `stack::end_stack_frame` - otherwise their values will be
/// references to garbage memory.
pub fn close_namespace() {
    SCOPE.with(|s| {
        let mut scope = s.borrow_mut();
        scope.pop().unwrap();
        debug_assert!(!scope.is_empty());
    });
}

/// Create a symbol, by returning a pointer to an existing one with
/// the same name or by allocating a new one if no such exists. This
/// is the *only legal way* to create a `Symbol` or a `SymRef` and it
/// garuntees that `Symbol`s with the same name will be `eq` (pointer
/// equal).
pub fn make_symbol(s: &[u8]) -> SymRef {
    let mut sym_heap = SYMBOLS_HEAP.lock().unwrap();
    if !sym_heap.contains_key(s) {
        let sym = Symbol::allocate(s);
        let sym = unsafe { SymRef::from_unchecked(sym) };
        let _insert_ref = sym_heap.insert(s.to_owned(), sym);
        debug_assert!(_insert_ref.is_none());
    }
    *(sym_heap.get(s).unwrap())
}

/// This method is called by `SymRef::evaluate`. It returns a
/// reference to the value of the symbol currently on the top of the
/// `SCOPE`, meaning that more recent local bindings are
/// preferred. This is the behavior you expect.
pub fn lookup_symbol(sym: SymRef) -> reference::Reference {
    SCOPE.with(|st| {
        let mut scope = st.borrow_mut();
        {
            use std::iter::DoubleEndedIterator;

            let mut iter = scope.iter();
            while let Some(n) = iter.next_back() {
                if let Some(r) = n.get_sym_ref(sym) {
                    return r;
                }
            }
        }
        scope[0].make_sym_ref(sym)
    })
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn deref_a_symbol() {
        let sym_name = b"my-sym";
        let sym = make_symbol(sym_name);
        assert_eq!((*sym).len(), sym_name.len());
        assert_eq!((*sym).as_ref(), sym_name);
    }
    #[test]
    fn symbols_are_eq() {
        let sym_name = b"any-symbol";
        let first = make_symbol(sym_name);
        let second = make_symbol(sym_name);
        assert_eq!(first, second);
    }
}
