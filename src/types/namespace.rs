use std::collections::HashMap;
use super::{Object, conversions, reference, symbol};

#[derive(Clone)]
pub enum Namespace {
    Heap {
        name: Option<Object>,
        table: HashMap<*const symbol::Symbol, *mut reference::HeapObject>,
    }
}

unsafe impl Send for Namespace {}
unsafe impl Sync for Namespace {}
