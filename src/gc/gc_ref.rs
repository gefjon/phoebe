//! The definition and impls for Phoebe's main internal reference
//! type, `GcRef`.

use prelude::*;
use std::{
    cmp, convert, fmt, hash, ops::{self, Deref}, ptr::NonNull,
};

/// This type is `Copy`, `Send` and `Sync`, and denotes a reference to
/// a garbage-collected object. It is very important that any such
/// objects be `gc_mark`'d during a garbage-collector sweep, or else
/// these references will dange. It is also important that, other than
/// their `GcMark`s, these objects are not mutated (soft exceptions
/// for `Namespace`s and for `unsafe` destructive functions like
/// `nreverse`
///
/// Because `NonNull<T>` is covariant over `T`, a lot of traits which
/// would be auto-impl'd or derived on a `GcRef(*mut T)` must be
/// implemented by hand.
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

impl<T> GcRef<T>
where
    T: GarbageCollected,
    Object: convert::From<Self>,
{
    pub fn should_dealloc(&self, m: usize) -> bool {
        T::should_dealloc(self, m)
    }
    pub fn gc_mark(mut self, m: usize) {
        T::gc_mark(&mut self, m)
    }
}

impl<T> Evaluate for GcRef<T>
where
    T: Evaluate,
    Object: convert::From<Self>,
{
    fn evaluate(&self) -> Object {
        self.deref().evaluate()
    }
    // fn eval_to_reference(&self) -> Result<Reference, EvaluatorError> {
    //     self.deref().eval_to_reference()
    // }
}
