//! This module exports `make_builtins`, which sources all builtin
//! functions and special forms. Phoebe is largely useless until that
//! function is called.

use crate::evaluator::eval_to_reference;
use crate::prelude::*;
use std::sync::{Once, ONCE_INIT};

static ONCE_BUILTINS: Once = ONCE_INIT;

#[macro_use]
mod macros;

mod error_handling;
mod math_builtins;
mod namespacing;

/// Any new thread which could be spawned before or during sourcing
/// builtins should call this function as its first act. Calling it
/// multiple times, either concurrently or in series, is safe and only
/// the first time will result in actual work being done. If another
/// thread is currently running `make_builtins_once`, a call to
/// `make_builtins_once` will sleep until that thread's call returns,
/// so any thread which calls `make_builtins_once` will garuntee that:
///
/// * builtins are sourced by the time `make_builtins_once` returns
///
/// * no UB will be caused by trying to do things while another thread
/// is setting up.
pub fn make_builtins_once() {
    ONCE_BUILTINS.call_once(make_builtins);
}

fn make_builtins() {
    info!("Making builtins.");
    special_forms! {
        "cond" (&rest clauses) -> {
            symbol_lookup::in_parent_env(|| -> Object {
                for clause in List::try_convert_from(*clauses)? {
                    let c: GcRef<Cons> = clause.try_convert_into()?;
                    let Cons { car, cdr, .. } = *c;
                    if bool::from(car.evaluate()?) {
                        let c: GcRef<Cons> = cdr.try_convert_into()?;
                        let Cons { car: cdrcar, cdr: tail, .. } = *c;
                        if !tail.nilp() {
                            return EvaluatorError::ImproperList.into();
                        }
                        return cdrcar.evaluate();
                    }
                }
                Object::nil()
            })
        };
        "if" (test then &rest elses) -> {
            symbol_lookup::in_parent_env(|| {
                if bool::from((*test).evaluate()?) {
                    (*then).evaluate()
                } else {
                    let mut res = Object::nil();
                    for clause in List::try_convert_from(*elses)? {
                        res = clause.evaluate()?;
                    }
                    res
                }
            })
        };
        "when" (test &rest clauses) -> {
            symbol_lookup::in_parent_env(|| {
                if bool::from((*test).evaluate()?) {
                    let mut res = Object::nil();
                    for clause in List::try_convert_from(*clauses)? {
                        res = clause.evaluate()?;
                    }
                    res
                } else {
                    Object::nil()
                }
            })
        };
        "unless" (test &rest clauses) -> {
            symbol_lookup::in_parent_env(|| {
                let mut res = (*test).evaluate()?;
                if !bool::from(res) {
                    for clause in List::try_convert_from(*clauses)? {
                        res = clause.evaluate()?;
                    }
                }
                res
            })
        };
        "let" (bindings &rest body) -> {
            let env = {
                let mut scope = Vec::new();

                symbol_lookup::in_parent_env(|| {
                    for binding_pair in List::try_convert_from(*bindings)? {
                        let c: GcRef<Cons> = binding_pair.try_convert_into()?;
                        let Cons { car: symbol, cdr, .. } = *c;
                        let c: GcRef<Cons> = cdr.try_convert_into()?;
                        let Cons { car: value, cdr: tail, .. } = *c;
                        if !tail.nilp() {
                            return EvaluatorError::ImproperList.into();
                        }
                        scope.push((
                            symbol.try_convert_into()?,
                            value.evaluate()?
                        ));
                    }
                    Object::nil()
                })?;

                Namespace::create_let_env(&scope)
            };

            let body = List::try_convert_from(*body)?;
            symbol_lookup::with_env(env, || {
                let mut res = Object::nil();
                for body_clause in body {
                    res = body_clause.evaluate()?;
                }
                res
            })
        };
        "lambda" (arglist &rest body) -> {
            Object::from(Function::allocate(
                Function::make_lambda(
                    (*arglist).try_convert_into()?,
                    (*body).try_convert_into()?,
                    symbol_lookup::scope_for_a_new_function()
                )?
            ))
        };
        "defvar" (name &optional value) -> {
            let sym = <GcRef<Symbol>>::try_convert_from(*name)?;
            let mut place = symbol_lookup::make_from_global_namespace(sym);
            if place.definedp() {
                Object::from(place)
            } else {
                let value: Object = *value;
                let value: Object = symbol_lookup::in_parent_env(|| {
                    if value.definedp() {
                        value.evaluate()
                    } else {
                        Object::uninitialized()
                    }
                })?;
                *place = value;
                Object::from(place)
            }
        };
        "boundp" (symbol) -> {
            let sym = <GcRef<Symbol>>::try_convert_from(*symbol)?;
            symbol_lookup::get_from_global_namespace(sym).is_some().into()
        };
        "defun" (name arglist &rest body) -> {
            let name = (*name).try_convert_into()?;
            let func = Object::from(Function::allocate(
                Function::make_lambda(
                    (*arglist).try_convert_into()?,
                    (*body).try_convert_into()?,
                    symbol_lookup::scope_for_a_new_function()
                )?.with_name(name)
            ));
            *(symbol_lookup::make_from_global_namespace(name)) = func;
            func
        };
        "setf" (place value) -> {
            let mut place: Reference = eval_to_reference(*place).try_convert_into()?;
            let value = *value;
            let value = symbol_lookup::in_parent_env(|| value.evaluate())?;
            *place = value;
            value
        };
        "quote" (x) -> {
            *x
        };
    };

    builtin_functions! {
        "cons" (first second) -> {
            Object::from(
                Cons::allocate(
                    Cons::new(*first, *second)
                )
            )
        };
        "list" (&rest elements) -> {
            *elements
        };
        "debug" (obj) -> {
            println!("{:?}", *obj);
            *obj
        };
    };

    namespacing::make_namespace_builtins();
    error_handling::make_error_builtins();
    math_builtins::make_math_builtins();

    info!("Finished making builtin functions.");
}
