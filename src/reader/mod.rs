use types::Object;
use std::iter::{Iterator, Peekable};

mod read_num;

#[derive(Fail, Debug)]
pub enum ReaderError {
    #[fail(display = "I have not yet implemented ReaderError")]
    DummyError,
}

pub fn read<I>(input: &mut Peekable<I>) -> Result<Option<Object>, ReaderError>
where I: Iterator<Item = u8>{
    unimplemented!()
}
