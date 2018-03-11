use std::{sync};
use types::{Object, reference};

static STACK_CAPACITY: usize = 128;

lazy_static! {
    pub static ref STACK: sync::Mutex<Vec<Object>> = {
        sync::Mutex::new(Vec::with_capacity(STACK_CAPACITY))
    };
}

pub fn current_stack_size() -> usize {
    STACK.lock().unwrap().len()
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

pub fn push(obj: Object) -> Result<reference::Reference, StackOverflowError> {
    use std::ops::IndexMut;
    
    let mut stack = STACK.lock().unwrap();
    let len = stack.len();
    if len == stack.capacity() {
        Err(StackOverflowError {
            stack_size: len,
            stack_capacity: stack.capacity(),
        })
    } else {
        stack.push(obj);
        Ok(reference::Reference::from(stack.index_mut(len)))
    }
}

pub fn pop() -> Result<Object, StackUnderflowError> {
    if let Some(obj) = STACK.lock().unwrap().pop() {
        Ok(obj)
    } else {
        Err(StackUnderflowError {})
    }
}
