use prelude::*;
use stack::StackUnderflowError;
use std::{convert, fmt, collections::HashMap};
use types::ConversionError;
use types::pointer_tagging::{ObjectTag, PointerTag};

lazy_static! {
    static ref FUNCTION_TYPE_NAME: GcRef<Symbol> = { symbol_lookup::make_symbol(b"function") };
    pub static ref OPTIONAL: GcRef<Symbol> = { symbol_lookup::make_symbol(b"&optional") };
    pub static ref REST: GcRef<Symbol> = { symbol_lookup::make_symbol(b"&rest") };
    pub static ref KEY: GcRef<Symbol> = { symbol_lookup::make_symbol(b"&key") };
}

enum ArgType {
    Mandatory,
    Optional,
    Rest,
    Key,
}

impl Function {
    fn count_stack_frame_length(arglist: List) -> Result<usize, ConversionError> {
        let mut ct = 0;
        for arg in arglist {
            let s = <GcRef<Symbol>>::try_convert_from(arg)?;
            if !(s == *REST || s == *OPTIONAL || s == *KEY) {
                ct += 1;
            }
        }
        Ok(ct)
    }
    pub fn make_lambda(
        arglist: List,
        body: List,
        env: GcRef<Namespace>,
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
        name: GcRef<Symbol>,
        arglist: List,
        body: &'static Fn() -> Result<Object, EvaluatorError>,
        env: GcRef<Namespace>,
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
        name: GcRef<Symbol>,
        arglist: List,
        body: &'static Fn() -> Result<Object, EvaluatorError>,
        env: GcRef<Namespace>,
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
    pub fn with_name(self, name: GcRef<Symbol>) -> Function {
        Function {
            name: Some(name),
            ..self
        }
    }
    pub fn call_to_reference(&self, args: List) -> Result<Reference, EvaluatorError> {
        let args = if self.should_evaluate_args() {
            let mut evaled_args = List::nil();
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
        Ok(Reference::try_convert_from(obj)?)
    }

    pub fn call(&self, args: List) -> Result<Object, EvaluatorError> {
        let args = if self.should_evaluate_args() {
            let mut evaled_args = List::nil();
            for a in args {
                evaled_args = evaled_args.push(a.evaluate()?);
            }
            evaled_args.reverse()
        } else {
            args
        };

        let env = self.build_env(args)?;
        let res = symbol_lookup::with_env(env, || {
            self.body.evaluate().map(|o| {
                if let Some(r) = Reference::maybe_from(o) {
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
    fn build_env(&self, mut args: List) -> Result<GcRef<Namespace>, EvaluatorError> {
        use stack::{end_stack_frame, push, ref_top};

        let mut arg_type = ArgType::Mandatory;
        let mut n_args: usize = 0;
        let mut stack_frame_length = 0;
        let mut symbol_lookup_buf = Vec::new();

        {
            let mut iter = self.arglist;
            'args: while let Some(arg) = iter.next() {
                let arg_sym: GcRef<Symbol> = arg.maybe_into().unwrap();
                if arg_sym == *OPTIONAL {
                    arg_type = ArgType::Optional;
                    continue;
                } else if arg_sym == *REST {
                    arg_type = ArgType::Rest;
                    continue;
                } else if arg_sym == *KEY {
                    arg_type = ArgType::Key;
                    continue;
                }
                match arg_type {
                    ArgType::Mandatory => {
                        if let Some(o) = args.next() {
                            if let Err(e) = push(o) {
                                end_stack_frame(stack_frame_length)?;
                                return Err(e.into());
                            } else {
                                n_args += 1;
                                stack_frame_length += 1;
                            }
                        } else {
                            end_stack_frame(stack_frame_length)?;
                            return Err(EvaluatorError::bad_args_count(self.arglist, n_args));
                        }
                        symbol_lookup_buf.push((arg_sym, ref_top()));
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
                        symbol_lookup_buf.push((arg_sym, ref_top()));
                    }
                    ArgType::Rest => {
                        if let Err(e) = push(Object::from(args)) {
                            end_stack_frame(stack_frame_length)?;
                            return Err(e.into());
                        } else {
                            n_args += args.count();
                            stack_frame_length += 1;
                            args = List::nil();
                        }
                        symbol_lookup_buf.push((arg_sym, ref_top()));
                    }
                    ArgType::Key => {
                        let mut pairs = HashMap::new();
                        'keys: loop {
                            let key = if let Some(k) = args.next() {
                                k
                            } else {
                                break 'keys;
                            };
                            let key = match key.try_convert_into() {
                                Ok(k) => k,
                                Err(e) => {
                                    end_stack_frame(stack_frame_length)?;
                                    return Err(e.into());
                                }
                            };
                            let val = if let Some(v) = args.next() {
                                v
                            } else {
                                end_stack_frame(stack_frame_length)?;
                                return Err(EvaluatorError::UnaccompaniedKey { key });
                            };
                            pairs.insert(key, val);
                        }
                        let s = arg_sym.with_colon_in_front();
                        let v = pairs.get(&s).cloned().unwrap_or_else(Object::uninitialized);
                        debug!("keyword pair {} -> {}", s, v);
                        if let Err(e) = push(v) {
                            end_stack_frame(stack_frame_length)?;
                            return Err(e.into());
                        }
                        stack_frame_length += 1;
                        symbol_lookup_buf.push((arg_sym, ref_top()));

                        for sym in iter {
                            debug!("{} is in the arglist while parsing keyword args", sym);
                            let sym: GcRef<Symbol> = sym.try_convert_into().unwrap();
                            let s: GcRef<Symbol> = sym.with_colon_in_front();
                            let v = pairs.get(&s).cloned().unwrap_or_else(Object::uninitialized);
                            debug!("keyword pair {} -> {}", s, v);
                            if let Err(e) = push(v) {
                                end_stack_frame(stack_frame_length)?;
                                return Err(e.into());
                            }
                            stack_frame_length += 1;
                            symbol_lookup_buf.push((sym, ref_top()));
                        }
                        break 'args;
                    }
                }
            }
        }

        Ok(Namespace::create_stack_env(&symbol_lookup_buf, self.env))
    }
    fn end_stack_frame(&self) -> Result<(), StackUnderflowError> {
        use stack::end_stack_frame;

        end_stack_frame(self.stack_frame_length)
    }
}

pub struct Function {
    gc_marking: GcMark,
    name: Option<GcRef<Symbol>>,
    arglist: List,
    body: FunctionBody,
    stack_frame_length: usize,
    env: GcRef<Namespace>,
}

enum FunctionBody {
    Source(List),
    Builtin(&'static Fn() -> Result<Object, EvaluatorError>),
    SpecialForm(&'static Fn() -> Result<Object, EvaluatorError>),
}

impl fmt::Display for FunctionBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FunctionBody::Source(ref l) => write!(f, "{}", l),
            FunctionBody::Builtin(_) => write!(f, "COMPILED BUILTIN"),
            FunctionBody::SpecialForm(_) => write!(f, "SPECIAL FORM"),
        }
    }
}

impl Evaluate for FunctionBody {
    fn evaluate(&self) -> Result<Object, EvaluatorError> {
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
    type ConvertFrom = Function;
    fn alloc_one_and_initialize(o: Self) -> ::std::ptr::NonNull<Self> {
        use std::heap::{Alloc, Heap};
        use std::ptr;
        let nn = Heap.alloc_one().unwrap();
        let p = nn.as_ptr();
        unsafe { ptr::write(p, o) };
        nn
    }
    fn my_marking(&self) -> &GcMark {
        &self.gc_marking
    }
    fn gc_mark_children(&mut self, mark: usize) {
        if let Some(symref) = self.name {
            symref.gc_mark(mark);
        }
        if let Some(c) = <GcRef<Cons>>::maybe_from(self.arglist) {
            c.gc_mark(mark);
        }
        if let FunctionBody::Source(b) = self.body {
            if let Some(c) = <GcRef<Cons>>::maybe_from(b) {
                c.gc_mark(mark);
            }
        }
    }
}

impl FromUnchecked<Object> for GcRef<Function> {
    unsafe fn from_unchecked(o: Object) -> Self {
        debug_assert!(Self::is_type(o));
        GcRef::from_ptr(Self::associated_tag().untag(o.0) as *mut Function)
    }
}

impl FromObject for GcRef<Function> {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Function
    }
    fn type_name() -> GcRef<Symbol> {
        *FUNCTION_TYPE_NAME
    }
}

impl convert::From<GcRef<Function>> for Object {
    fn from(f: GcRef<Function>) -> Object {
        Object::from_raw(ObjectTag::Function.tag(f.into_ptr() as u64))
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref s) = self.name {
            write!(f, "[function {}]", s)
        } else {
            write!(f, "[function ANONYMOUS]")
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref s) = self.name {
            write!(f, "[function {} {} -> {}]", s, self.arglist, self.body)
        } else {
            write!(f, "(lambda {} {})", self.arglist, self.body)
        }
    }
}
