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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
