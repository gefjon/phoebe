//! Builtin functions and special forms related to throwing, catching
//! and handling errors.

use prelude::*;
use std::ops::Try;

pub fn make_error_builtins() {
    builtin_functions! {
        "throw" (error) -> {
            Object::loud_error((*error).try_convert_into()?)
        };
        "error" (name &optional body) -> {
            Object::quiet_error(Error::user((*name).try_convert_into()?, *body))
        };
        "type-error" (wanted) -> {
            Object::quiet_error(Error::type_error((*wanted).try_convert_into()?))
        };
        "improper-list-error" () -> {
            Object::quiet_error(Error::improper_list())
        };
        "not-a-reference-error" () -> {
            Object::quiet_error(Error::cannot_be_referenced())
        };
    }
    special_forms! {
        "catch-error" (try bind &rest catch) -> {
            let bind: GcRef<Symbol> = (*bind).try_convert_into()?;
            let catch = List::try_convert_from(*catch)?;

            let mut env = None;
            match symbol_lookup::in_parent_env(|| {
                match (*try).evaluate().into_result() {
                    Ok(o) => o,
                    Err(e) => {
                        let quiet = Object::quiet_error(e);
                        env = Some(Namespace::create_let_env(&[(bind, quiet)]));
                        e.into()
                    }
                }
            }).into_result() {
                Ok(o) => o,
                Err(e) => {
                    symbol_lookup::with_env(env.unwrap(), || {
                        let mut res = Object::from(e);
                        for clause in catch {
                            res = clause.evaluate()?;
                        }
                        res
                    })
                }
            }
        };
    }
}
