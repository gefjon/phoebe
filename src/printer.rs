use stack;
/// Phoebe's printing facitlities are currently very bare-bones. In
/// the future, they may be expanded to interact with runtime config
/// like `print-readably` vs `pretty-print`, etc.
use types::Object;

pub fn print(obj: Object) -> Result<String, String> {
    use std::ops::Try;
    match obj.into_result() {
        Ok(o) => Ok(format!("{}", o)),
        Err(e) => Err(format!("{}", e)),
    }
}

pub unsafe fn print_from_stack() -> Result<String, String> {
    stack::with_stack(|s| {
        let to_print = s.pop().unwrap();
        print(to_print)
    })
}
