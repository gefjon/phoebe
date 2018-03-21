use types::Object;
use types::list::List;
use types::cons::Cons;
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
            symbol_lookup::add_heap_scope(&scope);
            let mut res = Ok(Object::nil());
            for body_clause in body {
                res = body_clause.evaluate();
                if res.is_err() {
                    symbol_lookup::close_namespace();
                    return res;
                }
            }
            symbol_lookup::close_namespace();
            res
        };
        "lambda" (arglist &rest body) -> {
            Ok(Function::allocate(
                Function::make_lambda(
                    (*arglist).try_into()?,
                    (*body).try_into()?,
                )?
            ))
        };

    };

    builtin_functions! {
        "list" (&rest elements) -> {
            Ok(*elements)
        };
    };
    info!("Finished making builtin functions.");
}
