use types::Object;
use std::iter::{Iterator, Peekable};

mod read_num;
use self::read_num::parse_to_object;

static WHITESPACE: &[u8] = &[b' ', b'\n', b'\t'];

#[derive(Fail, Debug)]
pub enum ReaderError {
    #[fail(display = "I have not yet implemented ReaderError")]
    DummyError,
}

fn peek<I>(input: &mut Peekable<I>) -> Option<u8>
where I: Iterator<Item = u8> {
    input.peek().cloned()
}

pub fn read<I>(input: &mut Peekable<I>) -> Result<Option<Object>, ReaderError>
where I: Iterator<Item = u8> {
    let mut buf = Vec::new();
    loop {
        match peek(input) {
            Some(w) if WHITESPACE.contains(&w) => {
                input.next();

                if buf.is_empty() {
                    continue;
                } else {
                    let obj = parse_to_object(&buf);
                    return Ok(Some(obj));
                }
            }
            Some(c) => {
                buf.push(c);
                continue;
            }
            None => {
                return Ok(None);
            }
        }
    }
}
