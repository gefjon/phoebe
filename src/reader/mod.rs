use types::{Object, list};
use std::iter::{Iterator, Peekable};

mod read_num;
use self::read_num::parse_to_object;

const WHITESPACE: &[u8] = &[b' ', b'\n', b'\t'];
const COMMENT_DESIGNATOR: u8 = b';';

#[derive(Fail, Debug)]
pub enum ReaderError {
    #[fail(display = "A list went unclosed")]
    UnclosedList,
    #[fail(display = "A spurious close-delimiter")]
    ExtraClose,
}

/// This method is analogous to `iter.next`, but it skips past
/// comments.
fn next<I>(input: &mut Peekable<I>) -> Option<u8>
where I: Iterator<Item = u8> {
    match input.next() {
        None => None,
        Some(c) if c == COMMENT_DESIGNATOR => {
            input.next();
            loop {
                match input.next() {
                    Some(b'\n') => { return next(input); }
                    Some(_) => { continue; }
                    None => { return None; }
                }
            }
        }
        Some(c) => Some(c),
    }
}

/// This method is a cheap hack around `Peekable.peek` because `peek`
/// returns a reference, even for `Copy` types. This method clones the
/// peeked value to make the borrow checker shut up, and also skips
/// past comments.
fn peek<I>(input: &mut Peekable<I>) -> Option<u8>
where I: Iterator<Item = u8> {
    match input.peek().cloned() {
        Some(c) if c == COMMENT_DESIGNATOR => {
            input.next();
            loop {
                match input.peek().cloned() {
                    Some(b'\n') => {
                        input.next();
                        return peek(input);
                    }
                    Some(_) => {
                        input.next();
                        continue;
                    }
                    None => { return None; }
                }
            }
        }
        Some(c) => Some(c),
        None => None,
    }
}

pub fn read<I>(input: &mut Peekable<I>) -> Result<Option<Object>, ReaderError>
where I: Iterator<Item = u8> {
    match peek(input) {
        Some(w) if WHITESPACE.contains(&w) => {
            next(input);
            read(input)
        }
        Some(b'"') => {
            next(input);
            Ok(Some(Object::from(read_string(input)?)))
        }
        Some(b')') => Err(ReaderError::ExtraClose),
        Some(b'(') => {
            next(input);
            Ok(Some(Object::from(read_list(input)?)))
        }
        Some(_) => Ok(Some(read_sym_or_num(input))),
        None => Ok(None),
    }
}

fn read_string<I>(_input: &mut Peekable<I>) -> Result<Object, ReaderError>
where I: Iterator<Item = u8> {
    unimplemented!()
}

/// This method reads bytes into a buffer until it hits whitespace or
/// a close-paren and then uses `read_num::parse_to_object` to convert
/// the buffer into an `Object`. If we parsed slices instead of an
/// iterator, we could skip the buffer and pass a slice of the input
/// to `parse_to_object`, but parsing slices would cause other
/// problems.
fn read_sym_or_num<I>(input: &mut Peekable<I>) -> Object
where I: Iterator<Item = u8> {
    let mut buf = Vec::new();
    loop {
        match peek(input) {
            Some(w) if WHITESPACE.contains(&w) => {
                next(input);
                debug_assert!(!buf.is_empty());
                return parse_to_object(&buf);
            }
            Some(b')') => {
                return parse_to_object(&buf);
            }
            Some(c) => {
                buf.push(c);
                next(input);
            }
            None => {
                debug_assert!(!buf.is_empty());
                return parse_to_object(&buf);
            }
        }
    }
}

// Notable behavior of this function: it expects that the opening
// paren will be consumed by `read`, and it itself consumes the
// closing paren.
/// This method recursively calls `read`, collects the resulting
/// objects into a vector, and then converts that vector into a
/// list. It would be more efficent to skip the vector and build the
/// list from the start.
fn read_list<I>(input: &mut Peekable<I>) -> Result<list::List, ReaderError>
where I: Iterator<Item = u8> {
    let mut objs = Vec::new();
    loop {
        match peek(input) {
            Some(w) if WHITESPACE.contains(&w) => {
                // We have already called `peek(input)`
                // so we don't have to worry about comments
                input.next();

                continue;
            }
            Some(b')') => {
                // We have alread called `peek(input)` so we don't
                // have to worry about comments
                input.next();
                
                return Ok(objs.iter().cloned().collect());
            }
            Some(_) => objs.push(read(input)?.unwrap()),
            None => { return Err(ReaderError::UnclosedList); }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn ignore_comments() {
        let input = b";; foobar \nw";
        assert_eq!(next(&mut input.iter().cloned().peekable()), Some(b'w'));
    }
    #[test]
    fn peek_past_comments() {
        let input = b";; foobar\nw";
        let iter = &mut input.iter().cloned().peekable();
        assert_eq!(peek(iter), Some(b'w'));
        assert_eq!(peek(iter), Some(b'w'));
    }
}
