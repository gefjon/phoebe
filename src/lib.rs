#![feature(box_leak)]
#![feature(specialization)]
#![feature(allocator_api)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate failure;
#[macro_use] extern crate failure_derive;

pub mod types;
mod symbol_lookup;
mod stack;
mod gc;
mod allocate;
pub mod reader;
pub mod evaluator;
pub mod printer;
pub mod repl;
mod builtins;

pub use repl::read_eval_print_loop;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
