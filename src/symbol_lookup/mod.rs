use types::{Object, reference, namespace};
use types::conversions::*;
use std::{sync};

type Scope = Vec<&'static mut namespace::Namespace>;

lazy_static! {
    static ref SYMBOLS_HEAP: sync::Mutex<Scope> = {
        sync::Mutex::new(Scope::new())
    };
}
