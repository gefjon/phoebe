use super::pointer_tagging::{ObjectTag, PointerTag};
use crate::prelude::*;
use crate::symbol_lookup::make_symbol;
use std::alloc::{Alloc, Global, Layout};
use std::ptr::NonNull;
use std::{convert, fmt, hash, mem, ptr, slice, str};

lazy_static! {
    static ref SYMBOL_TYPE_NAME: GcRef<Symbol> = { make_symbol(b"symbol") };
}

pub struct Symbol {
    gc_marking: GcMark,
    length: usize,
    head: u8,
}

impl hash::Hash for Symbol {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        AsRef::<[u8]>::as_ref(self).hash(state);
    }
}

impl GarbageCollected for Symbol {
    /// The understanding here is that `ConvertFrom` is a **valid**
    /// `*const [u8]`. This would be a `&[u8]` - its borrow only lasts
    /// the lifetime of `alloc_one_and_intitialize` and so any `&[u8]`
    /// is valid - but that would require it to be generic over the
    /// lifetime of the `&[u8]` and Generic Associated Types is very
    /// unstable.
    type ConvertFrom = *const [u8];

    fn alloc_one_and_initialize(text: *const [u8]) -> NonNull<Symbol> {
        use std::default::Default;

        let text = unsafe { &*text };

        let layout = Symbol::make_layout(text.len());

        #[cfg_attr(feature = "cargo-clippy", allow(cast_ptr_alignment))]
        let pointer = unsafe { Global.alloc(layout) }.unwrap().as_ptr() as *mut Symbol;
        let sym_ref = unsafe { &mut *pointer };
        sym_ref.gc_marking = GcMark::default();
        sym_ref.length = text.len();
        unsafe {
            ptr::copy_nonoverlapping(text.as_ptr(), sym_ref.pointer_mut(), text.len());
        }
        unsafe { NonNull::new_unchecked(pointer) }
    }
    unsafe fn deallocate(obj: GcRef<Self>) {
        let p = obj.into_ptr();
        ptr::drop_in_place((&mut *p).as_mut() as *mut [u8]);
        let layout = (&*p).my_layout();
        Global.dealloc(NonNull::new_unchecked(p as *mut u8), layout);
    }
    fn my_marking(&self) -> &GcMark {
        &self.gc_marking
    }
    fn gc_mark_children(&mut self, _: usize) {}
}

impl Symbol {
    pub fn with_colon_in_front(&self) -> GcRef<Symbol> {
        let mut vec = Vec::with_capacity(self.len() + 1);
        vec.push(b':');
        vec.extend_from_slice(self.as_ref());
        symbol_lookup::make_symbol(&vec)
    }
    fn is_self_evaluating(&self) -> bool {
        // The symbols `:` and `&` are *not* self-evaluating, but any
        // other symbols which start with `&` or `:` are.
        (self.len() > 1) && self.as_ref()[0] == b':' || self.as_ref()[0] == b'&'
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn my_layout(&self) -> Layout {
        Symbol::make_layout(self.len())
    }
    fn make_layout(len: usize) -> Layout {
        Layout::from_size_align(
            mem::size_of::<Symbol>() + len - 1,
            mem::align_of::<Symbol>(),
        )
        .unwrap()
    }
    pub fn len(&self) -> usize {
        self.length
    }
    fn pointer(&self) -> *const u8 {
        (&self.head) as *const u8
    }
    fn pointer_mut(&mut self) -> *mut u8 {
        (&mut self.head) as *mut u8
    }
}

impl convert::AsRef<[u8]> for Symbol {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.pointer(), self.len()) }
    }
}

impl convert::AsMut<[u8]> for Symbol {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.pointer_mut(), self.len()) }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            str::from_utf8(self.as_ref()).unwrap_or("##UNPRINTABLE##")
        )
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[symbol {}]", self)
    }
}

impl convert::From<GcRef<Symbol>> for Object {
    fn from(s: GcRef<Symbol>) -> Object {
        Object::from_raw(ObjectTag::Symbol.tag(s.into_ptr() as u64))
    }
}

impl FromObject for GcRef<Symbol> {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Symbol
    }
    fn type_name() -> GcRef<Symbol> {
        *SYMBOL_TYPE_NAME
    }
}

impl FromUnchecked<Object> for GcRef<Symbol> {
    unsafe fn from_unchecked(obj: Object) -> GcRef<Symbol> {
        debug_assert!(Self::is_type(obj));
        GcRef::from_ptr(Self::associated_tag().untag(obj.0) as *mut Symbol)
    }
}

impl Evaluate for Symbol {
    fn evaluate(&self) -> Object {
        let gc_r = unsafe { GcRef::from_ptr(self as *const Self as *mut Self) };
        if self.is_self_evaluating() {
            return Object::from(gc_r);
        }
        Object::from(symbol_lookup::lookup_symbol(gc_r)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::Object;
    #[test]
    fn tag_and_untag() {
        unsafe {
            let nonnull = 0xdead_beef as *mut Symbol;
            let obj = Object::from(GcRef::from_ptr(nonnull));
            assert_eq!(GcRef::from_ptr(nonnull), GcRef::from_unchecked(obj));
        }
    }
    #[test]
    fn symbol_type_name() {
        assert_eq!(format!("{}", GcRef::<Symbol>::type_name()), "symbol");
        assert_eq!(
            GcRef::<Symbol>::type_name(),
            crate::symbol_lookup::make_symbol(b"symbol")
        );
    }
}
