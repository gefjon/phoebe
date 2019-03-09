//! This module has two responsibilities: handing out references to
//! `Symbol`s and converting those references into `Reference`s to
//! `Object`s.

use crate::prelude::*;

use std::collections::HashMap;
use std::{cell, sync};

static GLOBAL_NAMESPACE_NAME: &[u8] = b"global-namespace";

lazy_static! {
    /// Because `Namespace`s can be used by several threads at once,
    /// and the garbage collector cannot see the contents of any
    /// threads' `ENV_STACK`, we count references to `Namespace`s in
    /// use. Each time a new stack frame is added, a call to
    /// `add_ref_to` accompanies it, and when the stack frame ends,
    /// there is a call to `remove_ref_to`. The garbage collector
    /// iterates through `ENV_REF_COUNTS`' `keys` to mark all used
    /// `Namespace`s.
    pub static ref ENV_REF_COUNTS: sync::Mutex<HashMap<GcRef<Namespace>, usize>> =
    { sync::Mutex::new(HashMap::new()) };

    /// The `SYMBOLS_HEAP` holds references to `Symbol`s in
    /// memory. Instead of directly calling
    /// `GarbageCollected::allocate`, `Symbol`s are constructed in the
    /// reader by `make_symbol`.
    pub static ref SYMBOLS_HEAP: sync::Mutex<HashMap<Vec<u8>, GcRef<Symbol>>> =
        { sync::Mutex::new(HashMap::new()) };
    static ref DEFAULT_GLOBAL_ENV: GcRef<Namespace> = {
        Namespace::allocate(
            Namespace::default().with_name(Object::from(make_symbol(GLOBAL_NAMESPACE_NAME))),
        )
    };
}

thread_local! {
    /// Each thread has an `ENV_STACK`, a stack of `Namespace`s. Each
    /// `Namespace` corresponds to either a function's stack frame or
    /// a `let` environment.
    static ENV_STACK: cell::RefCell<Vec<GcRef<Namespace>>> = {
        let g_e = default_global_env();
        add_ref_to(g_e);
        cell::RefCell::new(vec![g_e])
    };
}

#[derive(Fail, Debug)]
#[fail(display = "The symbol {} is unbound.", sym)]
pub struct UnboundSymbolError {
    pub sym: GcRef<Symbol>,
}

/// See `ENV_REF_COUNTS` for documentation.
fn add_ref_to(n: GcRef<Namespace>) {
    ENV_REF_COUNTS
        .lock()
        .unwrap()
        .entry(n)
        .and_modify(|n| *n += 1)
        .or_insert(1);
}

/// See `ENV_REF_COUNTS` for documentation.
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
        debug_assert!(_remove_res == Some(0));
    }
}

pub fn default_global_env() -> GcRef<Namespace> {
    *DEFAULT_GLOBAL_ENV
}

pub fn set_global_env(env: GcRef<Namespace>) {
    ENV_STACK.with(|s| {
        let stack: &mut Vec<GcRef<Namespace>> = &mut s.borrow_mut();
        stack[0] = env;
    })
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

/// Adds a `(SYMBOL VALUE)` pair to the global env.
pub fn add_to_global(sym: GcRef<Symbol>, obj: Object) {
    *(make_from_default_global_namespace(sym)) = obj;
}

pub fn gc_mark_scope(m: usize) {
    for env in ENV_REF_COUNTS.lock().unwrap().keys() {
        env.gc_mark(m);
    }
}

pub fn with_global_env<F>(env: GcRef<Namespace>, fun: F) -> Object
where
    F: FnOnce() -> Object,
{
    stack::push(Object::from(global_env()))?;
    set_global_env(env);
    let res = fun();
    set_global_env(unsafe { stack::pop()?.into_unchecked() });
    res
}

/// Executes a closure while `env` is on top of the stack, removing it
/// when finished.
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

/// Executes a closure in the `env` that is one step below the top of
/// the stack, removing it when done. This is used by special forms
/// like `cond`, which evaluate a form or forms - without using
/// `in_parent_env`,
///
/// ```lisp,text
/// (let ((x 3))
///   (cond
///     ((= x 4) 'three-equals-four)
///     (t x)))
/// ```
///
/// would error, as references to `x` within the `cond` block would be
/// undefined.
pub fn in_parent_env<F>(fun: F) -> Object
where
    F: FnOnce() -> Object,
{
    stack::push(Object::from({
        ENV_STACK.with(|s| {
            let mut stack = s.borrow_mut();
            debug_assert!(stack.len() > 1);
            stack.pop().unwrap()
        })
    }))?;
    let res = fun();
    let second_res = ENV_STACK.with(|s| -> Result<(), EvaluatorError> {
        s.borrow_mut()
            .push(unsafe { stack::pop()?.into_unchecked() });
        Ok(())
    });

    res?;
    second_res?;

    res
}

/// Create a symbol by returning a pointer to an existing one with the
/// same name or by allocating a new one if no such exists. This is
/// the *only legal way* to create a `Symbol` or a `GcRef<Symbol>` and
/// it garuntees that `Symbol`s with the same name will be `eq`
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

/// This method is called by `Symbol::evaluate`. It searches the
/// current lexical environment for a binding for `sym`, returning
/// `Err` if none exists.
pub fn lookup_symbol(sym: GcRef<Symbol>) -> Result<Reference, UnboundSymbolError> {
    current_env()
        .get_sym_ref(sym)
        .ok_or(UnboundSymbolError { sym })
}

/// Returns a reference to `sym`'s binding in `global_env()`, the
/// current global environment.
pub fn get_from_global_namespace(sym: GcRef<Symbol>) -> Option<Reference> {
    global_env().get_sym_ref(sym)
}

/// Returns a reference to `sym` in `global_env()`, inserting it if
/// none exists.
pub fn make_from_global_namespace(sym: GcRef<Symbol>) -> Reference {
    global_env().make_sym_ref(sym)
}

/// Like `make_from_global_namespace`, but always uses
/// `default_global_env`, even if the current global environment is
/// different. Builtins are always sourced into `default_global_env`,
/// so they use this function.
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
