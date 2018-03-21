use std::io::prelude::*;
use std::{convert, io};
use reader::read;
use evaluator::Evaluate;
use printer::print;
use builtins::make_builtins;

const PROMPT: &[u8] = b"phoebe> ";

#[derive(Fail, Debug)]
pub enum ReplError {
    #[fail(display = "IO error {}", _0)]
    IoError(io::Error),
}

impl convert::From<io::Error> for ReplError {
    fn from(e: io::Error) -> ReplError {
        ReplError::IoError(e)
    }
}

pub fn read_eval_print_loop<I, O, E>(
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
    make_builtins();
    let input_iter = &mut input.bytes().map(Result::unwrap).peekable();
    loop {
        if should_prompt {
            prompt(output)?;
        }
        match read(input_iter) {
            Err(e) => writeln!(error, "{}", e)?,
            Ok(None) => {
                return Ok(());
            }
            Ok(Some(obj)) => match obj.evaluate() {
                Err(e) => writeln!(error, "{}", e)?,
                Ok(obj) => writeln!(output, "{}", print(obj))?,
            },
        }
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

        read_eval_print_loop(&mut input, &mut output, &mut error, false).unwrap();
        if !error.is_empty() {
            panic!(
                "read_eval_print_loop errored: {}",
                str::from_utf8(&error).unwrap()
            );
        }
        assert_eq!(str::from_utf8(&output).unwrap(), "(1 2 3 4)\n");
    }
}
