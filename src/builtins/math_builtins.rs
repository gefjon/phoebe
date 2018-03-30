use prelude::*;

pub fn make_math_builtins() {
    builtin_functions! {
        "=" (&rest nums) -> {
            let mut nums = List::try_convert_from(*nums)?;
            if let Some(first) = nums.next() {
                let first: PhoebeNumber = first.try_convert_into()?;
                for n in nums {
                    let n = PhoebeNumber::try_convert_from(n)?;
                    if n != first {
                        return Ok(Object::from(false));
                    }
                }
            }
            Ok(Object::from(true))
        };
        "+" (&rest nums) -> {
            let mut result = PhoebeNumber::from(0);
            let nums = List::try_convert_from(*nums)?;
            for n in nums {
                let n = PhoebeNumber::try_convert_from(n)?;
                result += n;
            }
            Ok(result.into())
        };
        "*" (&rest nums) -> {
            let mut result = PhoebeNumber::from(1);
            let nums = List::try_convert_from(*nums)?;
            for n in nums {
                let n: PhoebeNumber = n.try_convert_into()?;
                result *= n;
            }
            Ok(Object::from(result))
        };
        "-" (number &rest others) -> {
            let mut number = PhoebeNumber::try_convert_from(*number)?;
            if others.nilp() {
                Ok(Object::from(-number))
            } else {
                let others = List::try_convert_from(*others)?;
                for n in others {
                    let n: PhoebeNumber = n.try_convert_into()?;
                    number -= n;
                }
                Ok(Object::from(number))
            }
        };
        "/" (number &rest others) -> {
            let mut number = PhoebeNumber::try_convert_from(*number)?;
            if others.nilp() {
                Ok(Object::from(number.recip()))
            } else {
                let others = List::try_convert_from(*others)?;
                for n in others {
                    let n: PhoebeNumber = n.try_convert_into()?;
                    number /= n;
                }
                Ok(Object::from(number))
            }
        };
    }
}
