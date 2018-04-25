//! Builtin functions and special forms related to namespacing.

use prelude::*;

pub fn make_namespace_builtins() {
    special_forms! {
        "make-namespace" (&key name contents parent) -> {
            let name = if (*name).definedp() {
                Some(<GcRef<Symbol>>::try_convert_from(*name)?)
            } else {
                None
            };
            let contents = if (*contents).definedp() {
                Some(List::try_convert_from(*contents)?)
            } else {
                None
            };
            let nmspc: Option<Result<Namespace, EvaluatorError>> =
                contents.map(|c| {
                    let mut pairs = Vec::<(GcRef<Symbol>, Object)>::new();

                    for pair in c {
                        let Cons { car: sym, cdr, .. } =
                            *(<GcRef<Cons>>::try_convert_from(pair)?);
                        let sym = <GcRef<Symbol>>::try_convert_from(sym)?;

                        let Cons { car: value, .. } =
                            *(<GcRef<Cons>>::try_convert_from(cdr)?);
                        let val = symbol_lookup::in_parent_env(|| value.evaluate())?;

                        pairs.push((sym, val));
                    }
                    Ok(pairs)
                })
                .map(|v| v.map(|v| v.iter()
                               .cloned()
                               .collect()));

            let mut nmspc = match nmspc {
                Some(n) => n?,
                None => Namespace::default(),
            };

            nmspc = nmspc
                .with_maybe_name(name.map(Object::from));

            if let Some(p) = <GcRef<Namespace>>::maybe_from(*parent) {
                nmspc = nmspc.with_parent(p);
            } else if !(parent.nilp()) {
                nmspc = nmspc.with_parent(symbol_lookup::global_env());
            }

            let nmspc = Object::from(Namespace::allocate(nmspc));

            if let Some(s) = name {
                let mut r = symbol_lookup::make_from_global_namespace(s);
                *r = nmspc;
            };

            Ok(nmspc)
        };
        "nref" (namespace symbol) -> {
            let mut namespace = <GcRef<Namespace>>::try_convert_from(
                Evaluate::evaluate(&*namespace)?
            )?;
            let symbol = <GcRef<Symbol>>::try_convert_from(*symbol)?;
            Ok(Object::from(namespace.make_sym_ref_search_parent(symbol)))
        };
        "with-namespace" (namespace &rest body) -> {
            let namespace = <GcRef<Namespace>>::try_convert_from(
                Evaluate::evaluate(&*namespace)?
            )?;
            let body = List::try_convert_from(*body)?;
            symbol_lookup::with_global_env(namespace, || {
                symbol_lookup::in_parent_env(|| {
                    let mut res = Object::nil();
                    for clause in body {
                        res = Evaluate::evaluate(&clause)?;
                    }
                    Ok(res)
                })
            })
        };
    }
}
