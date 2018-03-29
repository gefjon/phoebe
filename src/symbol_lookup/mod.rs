use prelude::*;

use std::collections::HashMap;
use std::{cell, sync};

static GLOBAL_NAMESPACE_NAME: &[u8] = b"global-namespace";

lazy_static! {
    pub static ref ENV_REF_COUNTS: sync::Mutex<HashMap<GcRef<Namespace>, usize>> =
        { sync::Mutex::new(HashMap::new()) };
    pub static ref SYMBOLS_HEAP: sync::Mutex<HashMap<Vec<u8>, GcRef<Symbol>>> =
        { sync::Mutex::new(HashMap::new()) };
    static ref DEFAULT_GLOBAL_ENV: GcRef<Namespace> = {
        Namespace::allocate(
            Namespace::default().with_name(Object::from(make_symbol(GLOBAL_NAMESPACE_NAME))),
        )
    };
}

thread_local! {
    static ENV_STACK: cell::RefCell<Vec<GcRef<Namespace>>> = {
        add_ref_to(*DEFAULT_GLOBAL_ENV);
        cell::RefCell::new(vec![*DEFAULT_GLOBAL_ENV])
    };
}

#[derive(Fail, Debug)]
#[fail(display = "The symbol {} is unbound.", sym)]
pub struct UnboundSymbolError {
    sym: GcRef<Symbol>,
}

fn add_ref_to(n: GcRef<Namespace>) {
    ENV_REF_COUNTS
        .lock()
        .unwrap()
        .entry(n)
        .and_modify(|n| *n += 1)
        .or_insert(1);
}

fn remove_ref_to(n: GcRef<Namespace>) {
    let mut ref_counts = ENV_REF_COUNTS.lock().unwrap();
    let should_remove = {
        let n_refs = ref_counts.get_mut(&n).unwrap();
        debug_assert!(*n_refs > 0);
        *n_refs -= 1;
        *n_refs == 0
    };
    if should_remove {
        let _remove_res = ref_counts.remove(&n);
        debug_assert!(_remove_res.is_some());
    }
}

pub fn default_global_env() -> GcRef<Namespace> {
    *DEFAULT_GLOBAL_ENV
}

pub fn current_env() -> GcRef<Namespace> {
    ENV_STACK.with(|s| {
        let stack: &Vec<GcRef<Namespace>> = &s.borrow();
        stack[stack.len() - 1]
    })
}

pub fn global_env() -> GcRef<Namespace> {
    ENV_STACK.with(|s| {
        let stack: &Vec<GcRef<Namespace>> = &s.borrow();
        stack[0]
    })
}

pub fn add_to_global(sym: GcRef<Symbol>, obj: Object) {
    *(make_from_default_global_namespace(sym)) = obj;
}

/// BUG: The `ENV_STACK` is thread local, but garbage collection is
/// done globally. This means that the garbage collector cannot mark
/// other threads' scopes and may deallocate them prematurely. This is
/// currently a non-issue because Phoebe is single-threaded, but in
/// the future could cause problems.
pub fn gc_mark_scope(m: usize) {
    for env in ENV_REF_COUNTS.lock().unwrap().keys() {
        env.gc_mark(m);
    }
}

pub fn with_env<F, T>(env: GcRef<Namespace>, fun: F) -> T
where
    F: FnOnce() -> T,
    T: Sized,
{
    {
        add_ref_to(env);
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
        });
        remove_ref_to(env);
    }
    res
}

/// Create a symbol, by returning a pointer to an existing one with
/// the same name or by allocating a new one if no such exists. This
/// is the *only legal way* to create a `Symbol` or a `GcRef<Symbol>`
/// and it garuntees that `Symbol`s with the same name will be `eq`
/// (pointer equal).
pub fn make_symbol(s: &[u8]) -> GcRef<Symbol> {
    let mut sym_heap = SYMBOLS_HEAP.lock().unwrap();
    if !sym_heap.contains_key(s) {
        let sym = Symbol::allocate(s);
        let _insert_ref = sym_heap.insert(s.to_owned(), sym);
        debug_assert!(_insert_ref.is_none());
    }
    *(sym_heap.get(s).unwrap())
}

/// This method is called by `Symbol::evaluate`. It returns a
/// reference to the value of the symbol currently on the top of the
/// `SCOPE`, meaning that more recent local bindings are
/// preferred. This is the behavior you expect.
pub fn lookup_symbol(sym: GcRef<Symbol>) -> Result<Reference, UnboundSymbolError> {
    current_env()
        .get_sym_ref(sym)
        .ok_or(UnboundSymbolError { sym })
}

pub fn get_from_global_namespace(sym: GcRef<Symbol>) -> Option<Reference> {
    global_env().get_sym_ref(sym)
}

pub fn make_from_global_namespace(sym: GcRef<Symbol>) -> Reference {
    global_env().make_sym_ref(sym)
}

pub fn make_from_default_global_namespace(sym: GcRef<Symbol>) -> Reference {
    default_global_env().make_sym_ref(sym)
}

/// The correct scope for a newly defined function is one step behind
/// the current scope - the current scope is either `lambda` or
/// `defun`'s scope.
pub fn scope_for_a_new_function() -> GcRef<Namespace> {
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
