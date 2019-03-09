//! Builtin functions related to mathematical and arithmetic ops.

use crate::prelude::*;

/// This function is called by `make_builtins`. It does no checking
/// for whether these functions have already been built, so calling it
/// in any other scenario will cause UB.
///
/// When adding new functions, if they accept only numbers as
/// arguments or accept no arguments and produce numbers, they should
/// be here. Otherwise, they should be in the base `make_builtins` or
/// in a seperate module.
pub fn make_math_builtins() {
    builtin_functions! {
        "=" (&rest nums) -> {
            let mut nums = List::try_convert_from(*nums)?;
            if let Some(first) = nums.next() {
                let first: PhoebeNumber = first.try_convert_into()?;
                for n in nums {
                    let n = PhoebeNumber::try_convert_from(n)?;
                    if n != first {
                        return Object::from(false);
                    }
                }
            }
            Object::from(true)
        };
        "+" (&rest nums) -> {
            let mut result = PhoebeNumber::from(0);
            let nums = List::try_convert_from(*nums)?;
            for n in nums {
                let n = PhoebeNumber::try_convert_from(n)?;
                result += n;
            }
            result.into()
        };
        "*" (&rest nums) -> {
            let mut result = PhoebeNumber::from(1);
            let nums = List::try_convert_from(*nums)?;
            for n in nums {
                let n: PhoebeNumber = n.try_convert_into()?;
                result *= n;
            }
            Object::from(result)
        };
        "-" (number &rest others) -> {
            let mut number = PhoebeNumber::try_convert_from(*number)?;
            if others.nilp() {
                Object::from(-number)
            } else {
                let others = List::try_convert_from(*others)?;
                for n in others {
                    let n: PhoebeNumber = n.try_convert_into()?;
                    number -= n;
                }
                Object::from(number)
            }
        };
        "/" (number &rest others) -> {
            let mut number = PhoebeNumber::try_convert_from(*number)?;
            if others.nilp() {
                Object::from(number.recip())
            } else {
                let others = List::try_convert_from(*others)?;
                for n in others {
                    let n: PhoebeNumber = n.try_convert_into()?;
                    number /= n;
                }
                Object::from(number)
            }
        };
    }
}
