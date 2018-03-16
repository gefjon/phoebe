use types::{Object, reference, namespace};
use types::symbol::{SymRef, Symbol};
use allocate::Allocate;
use types::conversions::*;
use std::{sync};
use std::collections::{HashMap};

type Scope = Vec<&'static mut namespace::Namespace>;

static GLOBAL_NAMESPACE_NAME: &[u8] = b"global-namespace";

lazy_static! {
    pub static ref SYMBOLS_HEAP: sync::Mutex<HashMap<Vec<u8>, SymRef>> = {
        sync::Mutex::new(HashMap::new())
    };
    pub static ref SCOPE: sync::Mutex<Scope> = {
        let global_namespace = namespace::Namespace::allocate(
            namespace::Namespace::default().with_name(
                Object::from(make_symbol(GLOBAL_NAMESPACE_NAME))
            )
        );
        sync::Mutex::new(vec![unsafe { global_namespace.into_unchecked() }])
    };
}

pub fn make_symbol(s: &[u8]) -> SymRef {
    let mut sym_heap = SYMBOLS_HEAP.lock().unwrap();
    if !sym_heap.contains_key(s) {
        let sym = Symbol::allocate(s);
        let sym = unsafe { SymRef::from_unchecked(sym) };
        let _insert_ref = sym_heap.insert(s.to_owned(), sym);
        debug_assert!(_insert_ref.is_none());
    }
    *(sym_heap.get(s).unwrap())
}
        
pub fn lookup_symbol(_s: SymRef) -> reference::Reference {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn deref_a_symbol() {
        let sym_name = b"my-sym";
        let sym = make_symbol(sym_name);
        assert_eq!((*sym).len(), sym_name.len());
        assert_eq!((*sym).as_ref(), sym_name);
    }
    #[test]
    fn symbols_are_eq() {
        let sym_name = b"any-symbol";
        let first = make_symbol(sym_name);
        let second = make_symbol(sym_name);
        assert_eq!(first, second);
    }
}
