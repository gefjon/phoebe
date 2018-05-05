use builtins::make_builtins_once;
use evaluator::eval_from_stack;
use printer::print_from_stack;
use reader::{read, ReaderError};
use stack::{self, StackOverflowError};
use std::io::prelude::*;
use std::{convert, io};

const PROMPT: &[u8] = b"phoebe> ";

#[derive(Fail, Debug)]
pub enum ReplError {
    #[fail(display = "IO error {}", _0)]
    IoError(io::Error),
    #[fail(display = "{}", _0)]
    StackOverflow(StackOverflowError),
}

impl convert::From<io::Error> for ReplError {
    fn from(e: io::Error) -> ReplError {
        ReplError::IoError(e)
    }
}

impl convert::From<StackOverflowError> for ReplError {
    fn from(e: StackOverflowError) -> ReplError {
        ReplError::StackOverflow(e)
    }
}

/// This is a public-facing method and is usually what you want - it
/// initializes, evaluates the input, and prints it. The only caveat
/// is that successive calls to this will result in repeated calls to
/// `initialize`, which is wasteful and potentially will squash
/// things. For repeated calls, instead use `initialize` once and
/// `read_eval_print_loop` as many times as required.
pub fn repl<I, O, E>(
    input: &mut I,
    output: &mut O,
    error: &mut E,
    should_prompt: bool,
) -> Result<(), ReplError>
where
    I: Read,
    O: Write,
    E: Write,
{
    make_builtins_once();
    read_eval_print_loop(input, output, error, should_prompt)
}

enum ReadResult {
    NoneRead,
    Ok,
    StackError(StackOverflowError),
    ReadError(ReaderError),
}

/// Repeatedly read, evaluate, and print from `input` into `output`,
/// signaling any errors into `error`, until `input` is empty. If
/// `should_prompt`, will print `phoebe> ` before each `read`. This is
/// called internally by `repl` and is exposed mostly for testing.
fn read_eval_print_loop<I, O, E>(
    input: &mut I,
    output: &mut O,
    error: &mut E,
    should_prompt: bool,
) -> Result<(), ReplError>
where
    I: Read,
    O: Write,
    E: Write,
{
    let input_iter = &mut input.bytes().map(Result::unwrap).peekable();
    loop {
        if should_prompt {
            prompt(output)?;
        }
        match stack::with_stack(|s| match read(input_iter) {
            Err(e) => ReadResult::ReadError(e),
            Ok(None) => ReadResult::NoneRead,
            Ok(Some(obj)) => {
                if let Err(e) = stack::make_stack_frame(s, &[obj]) {
                    ReadResult::StackError(e)
                } else {
                    ReadResult::Ok
                }
            }
        }) {
            ReadResult::NoneRead => {
                return Ok(());
            }
            ReadResult::Ok => {
                unsafe { eval_from_stack() }
                // eval_from_stack pushes its return value to the
                // stack, but without a frame_length. Adding that
                // frame_length turns it into the stack frame for
                // `print_from_stack`.
                match unsafe { print_from_stack() } {
                    Ok(o) => writeln!(output, "{}", o)?,
                    Err(e) => writeln!(error, "{}", e)?,
                }
            }
            ReadResult::ReadError(e) => {
                writeln!(error, "{}", e)?;
            }
            ReadResult::StackError(e) => {
                return Err(e.into());
            }
        }
    }
}

pub mod test_utilities {
    use super::*;
    use std::{convert, string};

    #[derive(Fail, Debug)]
    pub enum TestIOPairsError {
        #[fail(display = "Phoebe errored internally: {}", _0)]
        InternalError(String),
        #[fail(display = "Expected {} to yield {} but found {}", input, expected, found)]
        WrongOutput {
            input: String,
            found: String,
            expected: String,
        },
        #[fail(display = "Error converting output to utf-8: {}", _0)]
        StringUtf8Error(string::FromUtf8Error),
    }

    impl convert::From<string::FromUtf8Error> for TestIOPairsError {
        fn from(e: string::FromUtf8Error) -> TestIOPairsError {
            TestIOPairsError::StringUtf8Error(e)
        }
    }

    pub fn test_input_output_pairs(pairs: &[(&str, &str)]) -> Result<(), TestIOPairsError> {
        for &(input, output) in pairs {
            let mut input_buf: &[u8] = input.as_bytes();
            let mut output_buf = Vec::with_capacity(output.len());
            let mut error_buf = Vec::new();

            repl(&mut input_buf, &mut output_buf, &mut error_buf, false).unwrap();

            if !error_buf.is_empty() {
                return Err(TestIOPairsError::InternalError(String::from_utf8(
                    error_buf,
                )?));
            }
            if output_buf != output.as_bytes() {
                return Err(TestIOPairsError::WrongOutput {
                    input: String::from(input),
                    found: String::from_utf8(output_buf)?,
                    expected: String::from(output),
                });
            }
        }

        Ok(())
    }

    #[macro_export]
    /// This macro is used to test that inputs result in expected
    /// outputs. Usage:
    ///
    /// ```rust
    /// # #[macro_use] extern crate phoebe;
    /// # fn main() {
    /// test_pairs! {
    ///   "(+ 1 2)" => "3";
    ///   "(* 5 5)" => "25";
    ///   "(defun 1+ (n) (+ n 1))" => "[function 1+]";
    ///   "(1+ 3)" => "4";
    /// }
    /// # }
    /// ```
    ///
    /// Inputs are run in series in the same thread, but other calls
    /// to `test_pairs` from other threads will run concurrently, so
    /// it is best to use unique symbol names based on the name of the
    /// current test. See the `tests` directory for examples.
    macro_rules! test_pairs {
        ($($inp:expr => $out:expr);+ $(;)*) => {{
            if let Err(e) = $crate::repl::test_utilities::test_input_output_pairs(&[
                $(($inp, concat!($out, "\n")),)+
            ]) {
                panic!("{}", e);
            }
        }};
    }
}

fn prompt<O>(output: &mut O) -> Result<(), ReplError>
where
    O: Write,
{
    output.write_all(PROMPT)?;
    output.flush()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str;
    #[test]
    fn make_a_list() {
        let mut input: &[u8] = b"(list 1 2 3 4)";
        let mut output: Vec<u8> = Vec::new();
        let mut error: Vec<u8> = Vec::new();

        repl(&mut input, &mut output, &mut error, false).unwrap();
        if !error.is_empty() {
            panic!("repl errored: {}", str::from_utf8(&error).unwrap());
        }
        assert_eq!(str::from_utf8(&output).unwrap(), "(1 2 3 4)\n");
    }
}
