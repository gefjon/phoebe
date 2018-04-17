use prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use symbol_lookup::make_symbol;

static GENSYM_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

pub fn make_gensym() -> GcRef<Symbol> {
    make_symbol(format!("GENSYM-{}", GENSYM_COUNT.fetch_add(1, Ordering::Relaxed)).as_bytes())
}
