/// Phoebe's printing facitlities are currently very bare-bones. In
/// the future, they may be expanded to interact with runtime config
/// like `print-readably` vs `pretty-print`, etc.
use types::Object;

pub fn print(obj: Object) -> String {
    format!("{}", obj)
}
