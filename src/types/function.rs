use types::{cons, list, namespace, reference, symbol, Object};
use types::namespace::NamespaceRef;
use types::conversions::*;
use gc::{GarbageCollected, GcMark};
use symbol_lookup;
use types::pointer_tagging::{ObjectTag, PointerTag};
use std::{convert, fmt};
use evaluator::{self, Evaluate, EvaluatorError};
use stack::StackUnderflowError;

lazy_static! {
    static ref FUNCTION_TYPE_NAME: symbol::SymRef = { ::symbol_lookup::make_symbol(b"function") };
    pub static ref OPTIONAL: symbol::SymRef = { ::symbol_lookup::make_symbol(b"&optional") };
    pub static ref REST: symbol::SymRef = { ::symbol_lookup::make_symbol(b"&rest") };
}

enum ArgType {
    Mandatory,
    Optional,
    Rest,
}

impl Function {
    fn count_stack_frame_length(arglist: list::List) -> Result<usize, ConversionError> {
        let mut ct = 0;
        for arg in arglist {
            let s = symbol::SymRef::try_from(arg)?;
            if !(s == *REST || s == *OPTIONAL) {
                ct += 1;
            }
        }
        Ok(ct)
    }
    pub fn make_lambda(
        arglist: list::List,
        body: list::List,
        env: NamespaceRef,
    ) -> Result<Function, ConversionError> {
        Ok(Function {
            gc_marking: GcMark::default(),
            name: None,
            arglist,
            body: FunctionBody::Source(body),
            stack_frame_length: Function::count_stack_frame_length(arglist)?,
            env,
        })
    }
    pub fn make_special_form(
        name: symbol::SymRef,
        arglist: list::List,
        body: &'static Fn() -> Result<Object, EvaluatorError>,
        env: NamespaceRef,
    ) -> Result<Function, ConversionError> {
        Ok(Function {
            gc_marking: GcMark::default(),
            name: Some(name),
            arglist,
            body: FunctionBody::SpecialForm(body),
            stack_frame_length: Function::count_stack_frame_length(arglist)?,
            env,
        })
    }
    pub fn make_builtin(
        name: symbol::SymRef,
        arglist: list::List,
        body: &'static Fn() -> Result<Object, EvaluatorError>,
        env: NamespaceRef,
    ) -> Result<Function, ConversionError> {
        Ok(Function {
            gc_marking: GcMark::default(),
            name: Some(name),
            arglist,
            body: FunctionBody::Builtin(body),
            stack_frame_length: Function::count_stack_frame_length(arglist)?,
            env,
        })
    }
    pub fn with_name(self, name: symbol::SymRef) -> Function {
        Function {
            name: Some(name),
            ..self
        }
    }
    pub fn call_to_reference(
        &self,
        args: list::List,
    ) -> Result<reference::Reference, EvaluatorError> {
        let args = if self.should_evaluate_args() {
            let mut evaled_args = list::List::nil();
            for a in args {
                evaled_args = evaled_args.push(a.evaluate()?);
            }
            evaled_args
        } else {
            args
        };

        let env = self.build_env(args)?;
        let res = symbol_lookup::with_env(env, || self.body.evaluate());
        let second_res = self.end_stack_frame();

        let obj = second_res.map_err(EvaluatorError::from).and(res)?;
        Ok(obj.try_into()?)
    }

    pub fn call(&self, args: list::List) -> Result<Object, EvaluatorError> {
        let args = if self.should_evaluate_args() {
            let mut evaled_args = list::List::nil();
            for a in args {
                evaled_args = evaled_args.push(a.evaluate()?);
            }
            evaled_args
        } else {
            args
        };

        let env = self.build_env(args)?;
        let res = symbol_lookup::with_env(env, || {
            self.body.evaluate().map(|o| {
                if let Some(r) = reference::Reference::maybe_from(o) {
                    *r
                } else {
                    o
                }
            })
        });
        let second_res = self.end_stack_frame();

        second_res.map_err(EvaluatorError::from).and(res)
    }
    fn should_evaluate_args(&self) -> bool {
        if let FunctionBody::SpecialForm(_) = self.body {
            false
        } else {
            true
        }
    }
    fn build_env(&self, mut args: list::List) -> Result<NamespaceRef, EvaluatorError> {
        use stack::{end_stack_frame, push, ref_top};

        let mut arg_type = ArgType::Mandatory;
        let mut n_args: usize = 0;
        let mut stack_frame_length = 0;
        let mut symbol_lookup_buf = Vec::new();

        for arg in self.arglist {
            let arg_sym: symbol::SymRef = arg.maybe_into().unwrap();
            if arg_sym == *OPTIONAL {
                arg_type = ArgType::Optional;
                continue;
            } else if arg_sym == *REST {
                arg_type = ArgType::Rest;
                continue;
            }
            match arg_type {
                ArgType::Mandatory => {
                    if let Some(o) = args.next() {
                        if let Err(e) = push(o) {
                            end_stack_frame(n_args)?;
                            return Err(e.into());
                        } else {
                            n_args += 1;
                            stack_frame_length += 1;
                        }
                    } else {
                        end_stack_frame(stack_frame_length)?;
                        return Err(EvaluatorError::bad_args_count(self.arglist, n_args));
                    }
                }
                ArgType::Optional => {
                    let (o, narg) = if let Some(o) = args.next() {
                        (o, 1)
                    } else {
                        (Object::uninitialized(), 0)
                    };
                    if let Err(e) = push(o) {
                        end_stack_frame(stack_frame_length)?;
                        return Err(e.into());
                    } else {
                        n_args += narg;
                        stack_frame_length += 1;
                    }
                }
                ArgType::Rest => {
                    if let Err(e) = push(Object::from(args.reverse())) {
                        end_stack_frame(stack_frame_length)?;
                        return Err(e.into());
                    } else {
                        n_args += args.count();
                        stack_frame_length += 1;
                        args = list::List::nil();
                    }
                }
            }

            symbol_lookup_buf.push((arg_sym, ref_top()));
        }
        Ok(namespace::Namespace::create_stack_env(
            &symbol_lookup_buf,
            self.env,
        ))
    }
    fn end_stack_frame(&self) -> Result<(), StackUnderflowError> {
        use stack::end_stack_frame;

        end_stack_frame(self.stack_frame_length)
    }
}

pub struct Function {
    gc_marking: GcMark,
    name: Option<symbol::SymRef>,
    arglist: list::List,
    body: FunctionBody,
    stack_frame_length: usize,
    env: NamespaceRef,
}

enum FunctionBody {
    Source(list::List),
    Builtin(&'static Fn() -> Result<Object, evaluator::EvaluatorError>),
    SpecialForm(&'static Fn() -> Result<Object, evaluator::EvaluatorError>),
}

impl fmt::Display for FunctionBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FunctionBody::Source(l) => write!(f, "{}", l),
            FunctionBody::Builtin(_) => write!(f, "COMPILED BUILTIN"),
            FunctionBody::SpecialForm(_) => write!(f, "SPECIAL FORM"),
        }
    }
}

impl evaluator::Evaluate for Function {
    fn evaluate(&self) -> Result<Object, evaluator::EvaluatorError> {
        Ok(Object::from(self as *const Function as *mut Function))
    }
}

impl evaluator::Evaluate for FunctionBody {
    fn evaluate(&self) -> Result<Object, evaluator::EvaluatorError> {
        match *self {
            FunctionBody::Source(l) => {
                let mut res = Object::nil();
                for clause in l {
                    res = clause.evaluate()?;
                }
                Ok(res)
            }
            FunctionBody::Builtin(b) | FunctionBody::SpecialForm(b) => b(),
        }
    }
}

impl GarbageCollected for Function {
    fn my_marking(&self) -> &GcMark {
        &self.gc_marking
    }
    fn my_marking_mut(&mut self) -> &mut GcMark {
        &mut self.gc_marking
    }
    fn gc_mark_children(&mut self, mark: GcMark) {
        if let Some(symref) = self.name {
            symref.gc_mark(mark);
        }
        if let Some(c) = <&mut cons::Cons>::maybe_from(self.arglist) {
            c.gc_mark(mark);
        }
        if let FunctionBody::Source(b) = self.body {
            if let Some(c) = <&mut cons::Cons>::maybe_from(b) {
                c.gc_mark(mark);
            }
        }
    }
}

impl FromUnchecked<Object> for *mut Function {
    unsafe fn from_unchecked(o: Object) -> *mut Function {
        debug_assert!(<*mut Function>::is_type(o));
        <*mut Function>::associated_tag().untag(o.0) as *mut Function
    }
}

impl FromObject for *mut Function {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Function
    }
    fn type_name() -> symbol::SymRef {
        *FUNCTION_TYPE_NAME
    }
}

impl convert::From<*mut Function> for Object {
    fn from(f: *mut Function) -> Object {
        Object(ObjectTag::Function.tag(f as u64))
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(s) = self.name {
            write!(f, "[function {}]", s)
        } else {
            write!(f, "[function ANONYMOUS]")
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(s) = self.name {
            write!(f, "[function {} {} -> {}]", s, self.arglist, self.body)
        } else {
            write!(f, "(lambda {} {})", self.arglist, self.body)
        }
    }
}
