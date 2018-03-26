use types::Object;
use types::list::List;
use types::cons::Cons;
use types::namespace::Namespace;
use types::conversions::*;
use types::symbol::SymRef;
use evaluator::{Evaluate, EvaluatorError};
use symbol_lookup;

#[macro_use]
mod macros;

pub fn make_builtins() {
    info!("Making builtins.");
    special_forms! {
        "cond" (&rest clauses) -> {
            for clause in List::try_from(*clauses)? {
                let &Cons { car, cdr, .. } = clause.try_into()?;
                if bool::from(car.evaluate()?) {
                    let &Cons { car: cdrcar, cdr: tail, .. } = cdr.try_into()?;
                    if !tail.nilp() {
                        return Err(EvaluatorError::ImproperList);
                    }
                    return cdrcar.evaluate();
                }
            }
            Ok(Object::nil())
        };
        "let" (bindings &rest body) -> {
            let mut scope = Vec::new();
            for binding_pair in List::try_from(*bindings)? {
                let &Cons { car: symbol, cdr, .. } = binding_pair.try_into()?;
                let &Cons { car: value, cdr: tail, .. } = cdr.try_into()?;
                if !tail.nilp() {
                    return Err(EvaluatorError::ImproperList);
                }
                scope.push((
                    SymRef::try_from(symbol)?,
                    value.evaluate()?
                ));
            }
            let body = List::try_from(*body)?;
            let env = Namespace::create_let_env(&scope);
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
            Ok(Function::allocate(
                Function::make_lambda(
                    (*arglist).try_into()?,
                    (*body).try_into()?,
                    symbol_lookup::scope_for_a_new_function()
                )?
            ))
        };
        "defvar" (name &optional value) -> {
            let sym = SymRef::try_from(*name)?;
            let mut place = symbol_lookup::make_from_global_namespace(sym);
            if place.definedp() {
                Ok(Object::from(place))
            } else {
                let val = if (*value).definedp() {
                    value.evaluate()?
                } else {
                    Object::uninitialized()
                };
                *place = val;
                Ok(Object::from(place))
            }
        };
        "boundp" (symbol) -> {
            let sym = SymRef::try_from(*symbol)?;
            Ok(symbol_lookup::get_from_global_namespace(sym).is_some().into())
        };
        "defun" (name arglist &rest body) -> {
            let name = (*name).try_into()?;
            let func = Function::allocate(
                Function::make_lambda(
                    (*arglist).try_into()?,
                    (*body).try_into()?,
                    symbol_lookup::scope_for_a_new_function()
                )?.with_name(name)
            );
            *(symbol_lookup::make_from_global_namespace(name)) = func;
            Ok(func)
        };
        "setf" (place value) -> {
            let mut place = Evaluate::eval_to_reference(&*place)?;
            let value = Evaluate::evaluate(&*value)?;
            *place = value;
            Ok(value)
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
    info!("Finished making builtin functions.");
}
