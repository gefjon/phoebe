#![feature(try_trait)]
#![feature(never_type)]
#![feature(try_from)]
#![feature(specialization)]
#![feature(allocator_api)]

extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub(crate) mod allocate;
mod builtins;
pub(crate) mod evaluator;
pub(crate) mod gc;
pub mod gensym;
pub(crate) mod prelude;
pub(crate) mod printer;
pub(crate) mod reader;
pub mod repl;
mod stack;
pub mod symbol_lookup;
pub mod types;

pub use crate::repl::repl;
