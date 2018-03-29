use prelude::*;
use std::{cmp, convert, fmt, hash, ops::{self, Deref}, ptr::NonNull};

pub struct GcRef<T>(NonNull<T>);

impl<T> cmp::PartialEq for GcRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl<T> Clone for GcRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for GcRef<T> {}

impl<T> cmp::Eq for GcRef<T> {}

impl<T> hash::Hash for GcRef<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

impl<T> GcRef<T> {
    pub fn into_ptr(self) -> *mut T {
        self.0.as_ptr()
    }
    pub unsafe fn from_ptr(p: *mut T) -> GcRef<T> {
        GcRef(NonNull::new(p).unwrap())
    }
}

unsafe impl<T> Send for GcRef<T> {}
unsafe impl<T> Sync for GcRef<T> {}

impl<T> convert::Into<NonNull<T>> for GcRef<T> {
    fn into(self) -> NonNull<T> {
        self.0
    }
}

impl<T> AsRef<T> for GcRef<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T> AsMut<T> for GcRef<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

impl<T> Deref for GcRef<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> ops::DerefMut for GcRef<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T: fmt::Debug> fmt::Debug for GcRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[GcRef -> {:?}]", **self)
    }
}

impl<T: fmt::Display> fmt::Display for GcRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", **self)
    }
}

impl<T> convert::From<NonNull<T>> for GcRef<T> {
    fn from(p: NonNull<T>) -> GcRef<T> {
        GcRef(p)
    }
}

impl<T: GarbageCollected> GcRef<T> {
    pub fn should_dealloc(&self, m: usize) -> bool {
        T::should_dealloc(self, m)
    }
    pub fn gc_mark(mut self, m: usize) {
        T::gc_mark(&mut self, m)
    }
}

impl<T: Evaluate> Evaluate for GcRef<T> {
    fn evaluate(&self) -> Result<Object, EvaluatorError> {
        self.deref().evaluate()
    }
    fn eval_to_reference(&self) -> Result<Reference, EvaluatorError> {
        self.deref().eval_to_reference()
    }
}
