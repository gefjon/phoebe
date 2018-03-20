use types::{Object, symbol, list, cons};
use types::conversions::*;
use gc::{GcMark, GarbageCollected};
use evaluator;
use types::pointer_tagging::{ObjectTag, PointerTag};
use std::{convert, fmt};
use evaluator::{EvaluatorError, Evaluate};
use stack::StackUnderflowError;

lazy_static! {
    static ref FUNCTION_TYPE_NAME: symbol::SymRef = {
        ::symbol_lookup::make_symbol(b"function")
    };
    static ref OPTIONAL: symbol::SymRef = {
        ::symbol_lookup::make_symbol(b"&optional")
    };
    static ref REST: symbol::SymRef = {
        ::symbol_lookup::make_symbol(b"&rest")
    };
}

enum ArgType {
    Mandatory,
    Optional,
    Rest,
}

impl Function {
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
        self.put_args_on_stack(args)?;
        let res = self.body.evaluate();
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
    fn put_args_on_stack(&self, mut args: list::List) -> Result<(), EvaluatorError> {
        use stack::{push, end_stack_frame, ref_top};
        use symbol_lookup::add_namespace_to_scope;

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
                        (Object::nil(), 0)
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
                    if let Err(e) = push(Object::from(args)) {
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
        add_namespace_to_scope(&symbol_lookup_buf);
        Ok(())
    }
    fn end_stack_frame(&self) -> Result<(), StackUnderflowError> {
        use symbol_lookup::close_namespace;
        use stack::end_stack_frame;
        
        close_namespace();
        end_stack_frame(self.stack_frame_length)
    }
}

pub struct Function {
    gc_marking: GcMark,
    name: Option<symbol::SymRef>,
    arglist: list::List,
    body: FunctionBody,
    stack_frame_length: usize,
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
            FunctionBody::Builtin(b) => b(),
            FunctionBody::SpecialForm(b) => b(),
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
        match self.body {
            FunctionBody::Source(b) => {
                for obj in b {
                    obj.gc_mark(mark);
                }
            }
            _ => (),
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
