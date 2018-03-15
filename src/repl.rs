use std::io::prelude::*;
use std::{io, convert};
use reader::read;
use evaluator::Evaluate;
use printer::print;

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

pub fn read_eval_print_loop<I, O, E>(input: &mut I, output: &mut O, error: &mut E) -> Result<(), ReplError>
where I: Read,
      O: Write,
      E: Write {
    let input_iter = &mut input.bytes().map(Result::unwrap).peekable();
    loop {
        match read(input_iter) {
            Err(e) => writeln!(error, "{}", e)?,
            Ok(None) => {
                return Ok(());
            }
            Ok(Some(obj)) => {
                match obj.evaluate() {
                    Err(e) => writeln!(error, "{}", e)?,
                    Ok(obj) => {
                        match print(obj) {
                            Err(e) => writeln!(error, "{}", e)?,
                            Ok(s) => writeln!(output, "{}", s)?,
                        }
                    }
                }
            }
        }
    }
}
