use types::Object;

#[derive(Fail, Debug)]
pub enum PrinterError {
    #[fail(display = "I have not yet implemented PrinterError")]
    DummyError,
}

pub fn print(_obj: Object) -> Result<String, PrinterError> {
    unimplemented!()
}
