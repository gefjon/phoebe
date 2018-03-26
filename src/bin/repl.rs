extern crate env_logger;
extern crate phoebe;

fn main() {
    env_logger::init();
    use std::io::{stderr, stdin, stdout};
    let mut err = stderr();
    let mut input = stdin();
    let mut output = stdout();

    phoebe::repl::repl(&mut input, &mut output, &mut err, true).unwrap();
}
