use types::{reference, Object};
use types::symbol::{SymRef, Symbol};
use types::namespace::{Namespace, NamespaceRef};
use allocate::Allocate;
use types::conversions::*;
use std::{cell, sync};
use std::collections::HashMap;
use gc;

static GLOBAL_NAMESPACE_NAME: &[u8] = b"global-namespace";

lazy_static! {
    pub static ref SYMBOLS_HEAP: sync::Mutex<HashMap<Vec<u8>, SymRef>> =
        { sync::Mutex::new(HashMap::new()) };
    static ref DEFAULT_GLOBAL_ENV: NamespaceRef = {
        let n = Namespace::allocate(
            Namespace::default().with_name(Object::from(make_symbol(GLOBAL_NAMESPACE_NAME))),
        );
        unsafe { n.into_unchecked() }
    };
}

thread_local! {
    static ENV_STACK: cell::RefCell<Vec<NamespaceRef>> = {
        cell::RefCell::new(vec![*DEFAULT_GLOBAL_ENV])
    };
}

#[derive(Fail, Debug)]
#[fail(display = "The symbol {} is unbound.", sym)]
pub struct UnboundSymbolError {
    sym: SymRef,
}

pub fn default_global_env() -> NamespaceRef {
    *DEFAULT_GLOBAL_ENV
}

pub fn current_env() -> NamespaceRef {
    ENV_STACK.with(|s| {
        let stack: &Vec<NamespaceRef> = &s.borrow();
        stack[stack.len() - 1]
    })
}

pub fn global_env() -> NamespaceRef {
    ENV_STACK.with(|s| {
        let stack: &Vec<NamespaceRef> = &s.borrow();
        stack[0]
    })
}

pub fn add_to_global(sym: SymRef, obj: Object) {
    *(make_from_default_global_namespace(sym)) = obj;
}

/// BUG: The `ENV_STACK` is thread local, but garbage collection is
/// done globally. This means that the garbage collector cannot mark
/// other threads' scopes and may deallocate them prematurely. This is
/// currently a non-issue because Phoebe is single-threaded, but in
/// the future could cause problems.
pub fn gc_mark_scope(m: gc::GcMark) {
    ENV_STACK.with(|s| {
        for nmspc in s.borrow_mut().iter_mut() {
            nmspc.gc_mark(m);
        }
    });
}

pub fn with_env<F, T>(env: NamespaceRef, fun: F) -> T
where
    F: FnOnce() -> T,
    T: Sized,
{
    {
        ENV_STACK.with(|s| {
            s.borrow_mut().push(env);
        })
    }
    let res = fun();
    {
        ENV_STACK.with(|s| {
            let mut stack = s.borrow_mut();
            let _pop = stack.pop();
            debug_assert!(_pop.is_some());
            debug_assert!(_pop.unwrap() == env);
            debug_assert!(!stack.is_empty());
        })
    }
    res
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
pub fn lookup_symbol(sym: SymRef) -> Result<reference::Reference, UnboundSymbolError> {
    current_env()
        .get_sym_ref(sym)
        .ok_or(UnboundSymbolError { sym })
}

pub fn get_from_global_namespace(sym: SymRef) -> Option<reference::Reference> {
    global_env().get_sym_ref(sym)
}

pub fn make_from_global_namespace(sym: SymRef) -> reference::Reference {
    global_env().make_sym_ref(sym)
}

pub fn make_from_default_global_namespace(sym: SymRef) -> reference::Reference {
    default_global_env().make_sym_ref(sym)
}

/// The correct scope for a newly defined function is one step behind
/// the current scope - the current scope is either `lambda` or
/// `defun`'s scope.
pub fn scope_for_a_new_function() -> NamespaceRef {
    ENV_STACK.with(|s| {
        let scope = s.borrow();
        debug_assert!(scope.len() > 1);
        scope[scope.len() - 2]
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
