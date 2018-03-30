use prelude::*;
use std::{thread, sync::atomic::{AtomicBool, Ordering}, time::Duration};

#[macro_use]
mod macros;

mod math_builtins;

pub static STARTED_SOURCING_BUILTINS: AtomicBool = AtomicBool::new(false);
pub static FINISHED_SOURCING_BUILTINS: AtomicBool = AtomicBool::new(false);

pub fn make_builtins() {
    if STARTED_SOURCING_BUILTINS.swap(true, Ordering::AcqRel) {
        while !FINISHED_SOURCING_BUILTINS.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(10));
        }
        return;
    }
    info!("Making builtins.");
    special_forms! {
        "cond" (&rest clauses) -> {
            symbol_lookup::in_parent_env(|| {
                for clause in List::try_convert_from(*clauses)? {
                    let c: GcRef<Cons> = clause.try_convert_into()?;
                    let Cons { car, cdr, .. } = *c;
                    if bool::from(car.evaluate()?) {
                        let c: GcRef<Cons> = cdr.try_convert_into()?;
                        let Cons { car: cdrcar, cdr: tail, .. } = *c;
                        if !tail.nilp() {
                            return Err(EvaluatorError::ImproperList);
                        }
                        return cdrcar.evaluate();
                    }
                }
                Ok(Object::nil())
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
                            return Err(EvaluatorError::ImproperList);
                        }
                        scope.push((
                            symbol.try_convert_into()?,
                            value.evaluate()?
                        ));
                    }
                    Ok(())
                })?;

                Namespace::create_let_env(&scope)
            };

            let body = List::try_convert_from(*body)?;
            symbol_lookup::with_env(env, || {
                let mut res = Ok(Object::nil());
                for body_clause in body {
                    res = body_clause.evaluate();
                    if res.is_err() {
                        return res;
                    }
                }
                res
            })
        };
        "lambda" (arglist &rest body) -> {
            Ok(Object::from(Function::allocate(
                Function::make_lambda(
                    (*arglist).try_convert_into()?,
                    (*body).try_convert_into()?,
                    symbol_lookup::scope_for_a_new_function()
                )?
            )))
        };
        "defvar" (name &optional value) -> {
            let sym = <GcRef<Symbol>>::try_convert_from(*name)?;
            let mut place = symbol_lookup::make_from_global_namespace(sym);
            if place.definedp() {
                Ok(Object::from(place))
            } else {
                let value: Object = *value;
                let value: Object = symbol_lookup::in_parent_env(|| {
                    if value.definedp() {
                        value.evaluate()
                    } else {
                        Ok(Object::uninitialized())
                    }
                })?;
                *place = value;
                Ok(Object::from(place))
            }
        };
        "boundp" (symbol) -> {
            let sym = <GcRef<Symbol>>::try_convert_from(*symbol)?;
            Ok(symbol_lookup::get_from_global_namespace(sym).is_some().into())
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
            Ok(func)
        };
        "setf" (place value) -> {
            let mut place = Evaluate::eval_to_reference(&*place)?;
            let value = *value;
            let value = symbol_lookup::in_parent_env(|| value.evaluate())?;
            *place = value;
            Ok(value)
        };
        "quote" (x) -> {
            Ok(*x)
        };
    };

    builtin_functions! {
        "list" (&rest elements) -> {
            Ok(*elements)
        };
        "debug" (obj) -> {
            println!("{:?}", *obj);
            Ok(*obj)
        };
    };

    math_builtins::make_math_builtins();

    FINISHED_SOURCING_BUILTINS.store(true, Ordering::Release);
    info!("Finished making builtin functions.");
}
