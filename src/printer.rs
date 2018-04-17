use stack::{close_stack_frame, nth_arg};
/// Phoebe's printing facitlities are currently very bare-bones. In
/// the future, they may be expanded to interact with runtime config
/// like `print-readably` vs `pretty-print`, etc.
use types::Object;

pub fn print(obj: Object) -> String {
    format!("{}", obj)
}

pub unsafe fn print_from_stack() -> String {
    let to_print = nth_arg(0).unwrap();
    let o = print(*to_print);
    close_stack_frame();
    o
}
