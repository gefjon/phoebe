use std::{convert, fmt, mem, ops, ptr, slice, str};
use super::Object;
use super::pointer_tagging::{ObjectTag, PointerTag};
use super::conversions::*;
use allocate::{Allocate, DeallocError, Deallocate};
use gc::{GarbageCollected, GcMark};
use std::heap::{self, Alloc, Heap, Layout};
use symbol_lookup::make_symbol;

lazy_static! {
    static ref SYMBOL_TYPE_NAME: SymRef = {
        make_symbol(b"symbol")
    };
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct SymRef(*mut Symbol);

impl SymRef {
    pub fn gc_mark(self, m: GcMark) {
        unsafe { &mut *(self.0) }.gc_mark(m);
    }
}

impl AsRef<Symbol> for SymRef {
    fn as_ref(&self) -> &Symbol {
        unsafe { &*(self.0) }
    }
}

impl AsMut<Symbol> for SymRef {
    fn as_mut(&mut self) -> &mut Symbol {
        unsafe { &mut *(self.0) }
    }
}

impl ops::Deref for SymRef {
    type Target = Symbol;
    fn deref(&self) -> &Symbol {
        self.as_ref()
    }
}

impl convert::From<SymRef> for *mut Symbol {
    fn from(s: SymRef) -> *mut Symbol {
        s.0
    }
}

unsafe impl Send for SymRef {}
unsafe impl Sync for SymRef {}

impl convert::From<*mut Symbol> for SymRef {
    fn from(s: *mut Symbol) -> SymRef {
        SymRef(s)
    }
}

impl<'any> convert::From<&'any mut Symbol> for SymRef {
    fn from(s: &mut Symbol) -> SymRef {
        SymRef(s as *mut Symbol)
    }
}

impl convert::From<SymRef> for Object {
    fn from(s: SymRef) -> Object {
        Object::from(s.0 as *mut Symbol)
    }
}

impl fmt::Debug for SymRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", AsRef::<Symbol>::as_ref(self))
    }
}

impl fmt::Display for SymRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", AsRef::<Symbol>::as_ref(self))
    }
}

pub struct Symbol {
    gc_marking: GcMark,
    length: usize,
    head: u8,
}

impl Allocate<Symbol> for Symbol {
    fn alloc_one_and_initialize(_: Symbol) -> *mut Symbol {
        panic!("attempt to use default allocator on an unsized type!")
    }
}

impl<'any> Allocate<&'any [u8]> for Symbol {
    fn alloc_one_and_initialize(text: &[u8]) -> *mut Symbol {
        use std::default::Default;

        let layout = Symbol::make_layout(text.len());
        let pointer = match unsafe { Heap.alloc(layout) } {
            Ok(p) => p,
            Err(e) => heap::Heap.oom(e),
        } as *mut Symbol;
        let sym_ref = unsafe { &mut *pointer };
        sym_ref.gc_marking = GcMark::default();
        sym_ref.length = text.len();
        unsafe {
            ptr::copy_nonoverlapping(text.as_ptr(), sym_ref.pointer_mut(), text.len());
        }
        pointer
    }
}

impl Deallocate for Symbol {
    unsafe fn deallocate(p: *mut Symbol) -> Result<(), DeallocError> {
        if p.is_null() {
            Err(DeallocError::NullPointer)
        } else {
            ptr::drop_in_place((&mut *p).as_mut() as *mut [u8]);
            let layout = (&*p).my_layout();
            heap::Heap.dealloc(p as *mut u8, layout);
            Ok(())
        }
    }
}

impl GarbageCollected for Symbol {
    fn my_marking(&self) -> &GcMark {
        &self.gc_marking
    }
    fn my_marking_mut(&mut self) -> &mut GcMark {
        &mut self.gc_marking
    }
    fn gc_mark_children(&mut self, _: GcMark) {}
}

impl Symbol {
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
        ).unwrap()
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

impl convert::From<*mut Symbol> for Object {
    fn from(s: *mut Symbol) -> Object {
        Object(ObjectTag::Symbol.tag(s as u64))
    }
}

impl FromUnchecked<Object> for SymRef {
    unsafe fn from_unchecked(obj: Object) -> SymRef {
        SymRef(<*mut Symbol>::from_unchecked(obj))
    }
}

impl FromObject for SymRef {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Symbol
    }
    fn type_name() -> SymRef {
        *SYMBOL_TYPE_NAME
    }
}

impl FromUnchecked<Object> for *mut Symbol {
    unsafe fn from_unchecked(obj: Object) -> *mut Symbol {
        debug_assert!(<*mut Symbol>::is_type(obj));
        <*mut Symbol>::associated_tag().untag(obj.0) as *mut Symbol
    }
}

impl FromObject for *mut Symbol {
    type Tag = ObjectTag;
    fn associated_tag() -> ObjectTag {
        ObjectTag::Symbol
    }
    fn type_name() -> SymRef {
        *SYMBOL_TYPE_NAME
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use types::Object;
    use std::ptr;
    #[test]
    fn tag_and_untag() {
        let obj = Object::from(SymRef(ptr::null_mut()));
        assert_eq!(SymRef(ptr::null_mut()), unsafe {
            SymRef::from_unchecked(obj)
        });

        let nonnull = 0xdead_beef as *mut Symbol;
        let obj = Object::from(SymRef(nonnull));
        assert_eq!(SymRef(nonnull), unsafe { SymRef::from_unchecked(obj) });
    }
    #[test]
    fn symbol_type_name() {
        assert_eq!(format!("{}", SymRef::type_name()), "symbol");
    }
}
