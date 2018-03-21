extern crate phoebe;
extern crate env_logger;

fn main() {
    env_logger::init();
    use std::io::{stderr, stdin, stdout};
    let mut err = stderr();
    let mut input = stdin();
    let mut output = stdout();

    phoebe::repl::read_eval_print_loop(&mut input, &mut output, &mut err).unwrap();
}
