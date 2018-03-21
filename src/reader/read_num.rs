use types::Object;
use symbol_lookup::make_symbol;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Sign {
    Positive,
    Negative,
}

fn power_of_ten(e: i16) -> f64 {
    (10.0f64).powi(i32::from(e))
}

pub fn parse_to_object(s: &[u8]) -> Object {
    match parse_decimal(s) {
        ParseDecimalResult::Integer(i) => Object::from(i),
        ParseDecimalResult::Symbol(s) => Object::from(make_symbol(s)),
        ParseDecimalResult::Float(dec) => Object::from(dec.make_float()),
    }
}

#[derive(PartialEq, Eq, Debug)]
/// A sort of intermediate step between a `&[u8]` and an `f64`. The
/// various parts of a number have been seperated, but not yet
/// converted from `&[u8]`s into numbers.
struct DecimalFp<'a> {
    sign: Sign,
    integral: &'a [u8],
    fractional: &'a [u8],
    exp: i64,
}

impl<'a> DecimalFp<'a> {
    /// A simple way of converting a `DecimalFp` to a `f64`: parse the
    /// integral part into a float, parse the fractional part into a
    /// float and divide it by `10^f`, where `f` is the length of the
    /// fractional part in base 10, and then multiply the whole thing
    /// by `10^e`.
    fn make_float(mut self) -> f64 {
        simplify(&mut self);

        let integral = parse_float_from_bytes_unchecked(self.integral);
        let fractional = parse_float_from_bytes_unchecked(self.fractional)
            / power_of_ten(self.fractional.len() as i16);

        let combined = integral + fractional;

        combined * power_of_ten(self.exp as i16) * match self.sign {
            Sign::Positive => 1.0,
            Sign::Negative => -1.0,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum ParseDecimalResult<'a> {
    Integer(i32),
    Float(DecimalFp<'a>),
    Symbol(&'a [u8]),
}

/// Strings which represent integers are a subset of strings
/// representing floats, which in turn are a subset of
/// symbols. `parse_decimal` will return an integer iff passed an
/// integral with an optional leading sign; a float for an optional
/// leading sign, an integral, an optional `.` followed by a
/// fractional part, and an optional `e` or `E` followed by a legal
/// integer; or a symbol for any other string.
fn parse_decimal(input: &[u8]) -> ParseDecimalResult {
    debug_assert!(!input.is_empty());
    let (sign, s) = extract_sign(input);
    let (integral, s) = eat_digits(s);
    match s.first() {
        None => {
            debug_assert!(!integral.is_empty());
            let i = parse_num_from_bytes_unchecked(integral) as i32;
            match sign {
                Sign::Positive => ParseDecimalResult::Integer(i),
                Sign::Negative => ParseDecimalResult::Integer(-i),
            }
        }
        Some(&b'e') | Some(&b'E') => {
            if integral.is_empty() {
                ParseDecimalResult::Symbol(input)
            } else if let Some(exp) = parse_exp(&s[1..]) {
                ParseDecimalResult::Float(DecimalFp {
                    sign,
                    integral,
                    fractional: b"",
                    exp,
                })
            } else {
                ParseDecimalResult::Symbol(input)
            }
        }
        Some(&b'.') => {
            let (fractional, s) = eat_digits(&s[1..]);
            if integral.is_empty() && fractional.is_empty() {
                // we have parsed a symbol which starts with a '.'
                ParseDecimalResult::Symbol(input)
            } else {
                match s.first() {
                    None => ParseDecimalResult::Float(DecimalFp {
                        sign,
                        integral,
                        fractional,
                        exp: 0,
                    }),
                    Some(&b'e') | Some(&b'E') => {
                        if let Some(exp) = parse_exp(&s[1..]) {
                            ParseDecimalResult::Float(DecimalFp {
                                sign,
                                integral,
                                fractional,
                                exp,
                            })
                        } else {
                            ParseDecimalResult::Symbol(input)
                        }
                    }
                    Some(_) => ParseDecimalResult::Symbol(input),
                }
            }
        }
        Some(_) => ParseDecimalResult::Symbol(input),
    }
}

fn extract_sign(s: &[u8]) -> (Sign, &[u8]) {
    match s.first() {
        Some(&b'-') => (Sign::Negative, &s[1..]),
        Some(&b'+') => (Sign::Positive, &s[1..]),
        Some(_) | None => (Sign::Positive, s),
    }
}

fn parse_exp(s: &[u8]) -> Option<i64> {
    let (sign, s) = extract_sign(s);
    let (mut digits, trailing) = eat_digits(s);
    if !trailing.is_empty() {
        return None;
    }
    if digits.is_empty() {
        return None;
    }

    // This loop eats leading '0's from `digits`
    while digits.first() == Some(&b'0') {
        digits = &digits[1..0];
    }

    if digits.len() >= 18 {
        // The smart thing to do here would be what `libcore` does:
        // create `0.0` if `sign` is negative or `infinity` if sign is
        // positive.
        panic!("We don't actually handle parsing very large or very small numbers!");
    }

    let abs_exp = parse_num_from_bytes_unchecked(digits);
    let e = match sign {
        Sign::Positive => abs_exp as i64,
        Sign::Negative => -(abs_exp as i64),
    };
    Some(e)
}

fn parse_float_from_bytes_unchecked(s: &[u8]) -> f64 {
    let mut result = 0.0;
    for &c in s {
        result = result * 10.0 + f64::from(c - b'0');
    }
    result
}

fn parse_num_from_bytes_unchecked(s: &[u8]) -> u64 {
    let mut result = 0;
    for &c in s {
        result = result * 10 + u64::from(c - b'0');
    }
    result
}

fn eat_digits(s: &[u8]) -> (&[u8], &[u8]) {
    let mut i = 0;
    while i < s.len() && b'0' <= s[i] && s[i] <= b'9' {
        i += 1;
    }
    (&s[..i], &s[i..])
}

/// This method removes unneeded leading and trailing zeroes. My
/// belief is that fewer significant figures => less floating-point
/// error.
fn simplify(decimal: &mut DecimalFp) {
    let is_zero = &|&&d: &&u8| -> bool { d == b'0' };

    // First, count the leading zeroes in the integral part and remove them
    let leading_zeros = decimal.integral.iter().take_while(is_zero).count();
    decimal.integral = &decimal.integral[leading_zeros..];

    // Next, count the trailing zeroes in the fractional part and remove them
    let trailing_zeros = decimal.fractional.iter().rev().take_while(is_zero).count();
    let end = decimal.fractional.len() - trailing_zeros;
    decimal.fractional = &decimal.fractional[..end];

    if decimal.integral.is_empty() {
        // If the integral is zero and the fractional has leading
        // zeros, it is safe to remove those zeroes and adjust the
        // exponent.
        let leading_zeros = decimal.fractional.iter().take_while(is_zero).count();
        decimal.fractional = &decimal.fractional[leading_zeros..];
        decimal.exp -= leading_zeros as i64;
    } else if decimal.fractional.is_empty() {
        // If the fractional is zero and the integral has trailing
        // zeroes, it is safe to remove those zeroes and adjust the
        // exponent.
        let trailing_zeros = decimal.integral.iter().rev().take_while(is_zero).count();
        let end = decimal.integral.len() - trailing_zeros;
        decimal.integral = &decimal.integral[..end];
        decimal.exp += trailing_zeros as i64;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use types::Object;
    /// We accept an error of `::std::f64::EPSILON * lhs`, which, in
    /// theory, means that the exact value or the next or previous
    /// floats are all acceptable.
    fn equal_enough(lhs: f64, rhs: f64) -> bool {
        (lhs - rhs).abs() < (lhs * ::std::f64::EPSILON)
    }
    #[test]
    fn parse_decimals() {
        let res = parse_decimal(b"1.23");
        assert_eq!(
            res,
            ParseDecimalResult::Float(DecimalFp {
                sign: Sign::Positive,
                integral: b"1",
                fractional: b"23",
                exp: 0,
            })
        );

        let res = parse_decimal(b"100");
        assert_eq!(res, ParseDecimalResult::Integer(100));

        let res = parse_decimal(b"1E100");
        assert_eq!(
            res,
            ParseDecimalResult::Float(DecimalFp {
                sign: Sign::Positive,
                integral: b"1",
                fractional: b"",
                exp: 100,
            })
        );

        let res = parse_decimal(b"-10e-2");
        assert_eq!(
            res,
            ParseDecimalResult::Float(DecimalFp {
                sign: Sign::Negative,
                integral: b"10",
                fractional: b"",
                exp: -2,
            })
        );
    }
    #[test]
    fn parse_one() {
        let res = parse_to_object(b"1");
        assert_eq!(res, Object::from(1i32));
    }
    #[test]
    fn parse_a_float() {
        let res = parse_to_object(b"1.23");
        assert_eq!(res, Object::from(1.23f64));
    }
    #[test]
    fn parse_large_float() {
        let res = parse_to_object(b"12345678.910e11");
        assert_eq!(res, Object::from(12345678.910e11));
    }
    #[test]
    /// This method actually tests `f64.powi` - it turns out that
    /// `10.0.powi` is accurate enough for `equal_enough` but not
    /// accurate enough for `==`. If reducing floating-point error
    /// were a goal, I would probably do what `libcore` does and store
    /// a big array full of every possible power of 10 in memory.
    fn powers_of_ten() {
        assert!(equal_enough(power_of_ten(5), 1e5));
        assert!(equal_enough(power_of_ten(100), 1e100));
    }
}
