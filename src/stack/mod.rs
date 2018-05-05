use prelude::*;
use std::{
    borrow::BorrowMut, collections::HashMap, ops::IndexMut,
    sync::{
        atomic::{AtomicUsize, Ordering}, Mutex, RwLock,
    },
};

const STACK_CAPACITY: usize = 128;

thread_local! {
    static STACK_KEY: usize = {
        STACK_NUMBER.fetch_add(1, Ordering::Relaxed)
    };
}

lazy_static! {
    pub static ref STACK_NUMBER: AtomicUsize = { AtomicUsize::new(0) };
    pub static ref STACKS: RwLock<HashMap<usize, Mutex<Vec<Object>>>> =
        { RwLock::new(HashMap::new()) };
}

#[derive(Fail, Debug)]
#[fail(
    display = "Attempted to reference argument {} but only found {}.",
    attempted_index,
    stack_frame_length
)]
pub struct ArgIndexError {
    pub attempted_index: usize,
    pub stack_frame_length: usize,
}

pub fn make_stack_frame(s: &mut Vec<Object>, objs: &[Object]) -> Result<(), StackOverflowError> {
    for &obj in objs {
        push_to_vec_checked(s, obj)?;
    }
    push_to_vec_checked(s, objs.len().into())?;
    Ok(())
}

pub fn nth_arg(n: usize) -> Result<Reference, ArgIndexError> {
    with_stack(|s| {
        let n_args: usize = unsafe {
            (*(s.last().expect("Call to nth_arg while the stack is empty"))).into_unchecked()
        };
        if n >= n_args {
            return Err(ArgIndexError {
                attempted_index: n,
                stack_frame_length: n_args,
            });
        }
        let highest_idx_of_stack_frame = s.len() - 1;
        let lowest_idx_of_stack_frame = highest_idx_of_stack_frame - n_args;
        Ok((&mut s[lowest_idx_of_stack_frame + n]).into())
    })
}

pub fn close_stack_frame_and_return(ret_val: Object) {
    with_stack(|s| {
        let n_args: usize = unsafe { s.pop().unwrap().into_unchecked() };
        for _ in 0..n_args {
            s.pop().unwrap();
        }
        s.push(ret_val);
    })
}

pub fn with_stack<F, R>(fun: F) -> R
where
    F: FnOnce(&mut Vec<Object>) -> R,
{
    let k = STACK_KEY.with(|k| *k);
    {
        if let Some(m) = STACKS.read().unwrap().get(&k) {
            return fun(m.lock().unwrap().borrow_mut());
        }
    }
    {
        STACKS
            .write()
            .unwrap()
            .insert(k, Mutex::new(Vec::with_capacity(STACK_CAPACITY)));
    }
    if let Some(m) = STACKS.read().unwrap().get(&k) {
        fun(m.lock().unwrap().borrow_mut())
    } else {
        unreachable!()
    }
}

/// Returns a `Reference` pointing to the current top element of the
/// `STACK`. This is useful when creating local bindings - push a
/// value and immediately reference it.
///
/// Future improvement: A single method which combines `push` and
/// `ref_top` with only one call to `with_stack`
pub fn ref_top() -> Reference {
    with_stack(|stack| {
        if stack.is_empty() {
            panic!("Attempt to reference an empty stack");
        }
        let i = stack.len() - 1;
        (&mut stack[i]).into()
    })
}

/// BUG: The `STACK` is thread local, but garbage collection is done
/// globally. This means that the garbage collector cannot mark other
/// threads' stacks and may deallocate them prematurely.
pub fn gc_mark_stack(m: usize) {
    for stack in STACKS.read().unwrap().values() {
        for obj in stack.lock().unwrap().iter() {
            obj.gc_mark(m)
        }
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Stack overflow after {} elements with capacity {}", stack_size, stack_capacity)]
pub struct StackOverflowError {
    stack_size: usize,
    stack_capacity: usize,
}

#[derive(Fail, Debug)]
#[fail(display = "Attempt to pop off an empty stack.")]
pub struct StackUnderflowError {}

/// Attempts to push to a vector, returning the index of the newly
/// pushed element if successful
pub fn push_to_vec_checked<T>(v: &mut Vec<T>, i: T) -> Result<usize, StackOverflowError> {
    let len = v.len();
    let cap = v.capacity();
    if len == cap {
        Err(StackOverflowError {
            stack_size: len,
            stack_capacity: cap,
        })
    } else {
        v.push(i);
        Ok(len)
    }
}

/// It's bad if the stack gets realloc'd - all our outstanding
/// `Reference`s to the stack are suddenly invalid - so this method
/// checks that a `push` will not realloc and returns an error if it
/// will.
pub fn push(obj: Object) -> Result<Reference, StackOverflowError> {
    with_stack(|stack| {
        let idx = push_to_vec_checked(stack, obj)?;
        Ok(Reference::from(stack.index_mut(idx)))
    })
}

/// This method maps the `None` case of `Vec.pop`, which represents an
/// empty `Vec`, to an error - trying to `pop` off an empty stack is a
/// serious problem.
pub fn pop() -> Result<Object, StackUnderflowError> {
    with_stack(|s| s.pop().ok_or(StackUnderflowError {}))
}

/// Given a `length`, pop that many items off the stack. This is
/// called when ending local scopes to remove their values all at
/// once. This should be called *after* its corresponding
/// `symbol_lookup::close_namespace`.
pub fn end_stack_frame(length: usize) -> Result<(), StackUnderflowError> {
    for _ in 0..length {
        pop()?;
    }
    Ok(())
}
