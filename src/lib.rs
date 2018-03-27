#![feature(specialization)]
#![feature(allocator_api)]

extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod allocate;
mod builtins;
pub mod evaluator;
mod gc;
pub mod printer;
pub mod reader;
pub mod repl;
mod stack;
mod symbol_lookup;
pub mod types;

pub use repl::repl;
