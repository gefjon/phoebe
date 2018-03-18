use types::{Object, symbol, list, cons};
use types::conversions::*;
use gc::{GcMark, GarbageCollected};
use evaluator;
use types::pointer_tagging::{ObjectTag, PointerTag};
use std::{convert, fmt};

lazy_static! {
    static ref FUNCTION_TYPE_NAME: symbol::SymRef = {
        ::symbol_lookup::make_symbol(b"function")
    };
}

pub struct Function {
    gc_marking: GcMark,
    name: Option<symbol::SymRef>,
    arglist: list::List,
    body: FunctionBody,
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
