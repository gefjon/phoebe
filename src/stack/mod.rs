use std::cell;
use types::Object;
use types::reference::Reference;

const STACK_CAPACITY: usize = 128;

thread_local! {
    static STACK: cell::RefCell<Vec<Object>> = {
        cell::RefCell::new(Vec::with_capacity(STACK_CAPACITY))
    };
}

/// Returns a `Reference` pointing to the current top element of the
/// `STACK`. This is useful when creating local bindings - push a
/// value and immediately reference it.
///
/// Future improvement: A single method which combines `push` and
/// `ref_top` with only one call to `STACK.with`
pub fn ref_top() -> Reference {
    STACK.with(|s| {
        let mut stack = s.borrow_mut();
        if stack.is_empty() {
            panic!("Attempt to reference an empty stack");
        }
        let i = stack.len() - 1;
        stack.get_mut(i).unwrap().into()
    })
}

/// BUG: The `STACK` is thread local, but garbage collection is done
/// globally. This means that the garbage collector cannot mark other
/// threads' stacks and may deallocate them prematurely.
pub fn gc_mark_stack(m: usize) {
    STACK.with(|s| {
        for obj in s.borrow().iter() {
            obj.gc_mark(m);
        }
    })
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

/// It's bad if the stack gets realloc'd - all our outstanding
/// `Reference`s to the stack are suddenly invalid - so this method
/// checks that a `push` will not realloc and returns an error if it
/// will.
pub fn push(obj: Object) -> Result<Reference, StackOverflowError> {
    use std::ops::IndexMut;

    STACK.with(|s| {
        let mut stack = s.borrow_mut();
        let len = stack.len();
        if len == stack.capacity() {
            Err(StackOverflowError {
                stack_size: len,
                stack_capacity: stack.capacity(),
            })
        } else {
            stack.push(obj);
            Ok(Reference::from(stack.index_mut(len)))
        }
    })
}

/// This method maps the `None` case of `Vec.pop`, which represents an
/// empty `Vec`, to an error - trying to `pop` off an empty stack is a
/// serious problem.
pub fn pop() -> Result<Object, StackUnderflowError> {
    STACK.with(|s| s.borrow_mut().pop().ok_or(StackUnderflowError {}))
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
