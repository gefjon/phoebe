#[macro_use]
extern crate phoebe;

use phoebe::repl::repl;
use phoebe::symbol_lookup::make_symbol;
use phoebe::types::error::EvaluatorError;

#[test]
fn throw_an_error() {
    let mut output = String::new();
    let expected_error = format!(
        "{}\n",
        EvaluatorError::user(
            make_symbol(b"some-error"),
            make_symbol(b"error-description").into()
        )
    );
    let mut error = String::with_capacity(expected_error.len());

    let mut input: &[u8] =
        "(throw (error (quote some-error) (quote error-description)))".as_bytes();

    repl(
        &mut input,
        unsafe { output.as_mut_vec() },
        unsafe { error.as_mut_vec() },
        false,
    )
    .unwrap();

    assert_eq!(error, expected_error);
    assert!(output.is_empty());
}

#[test]
fn build_error_without_throw() {
    test_pairs! {
        "(error (quote some-error) (quote error-description))" => "some-error: error-description";
    }
}

#[test]
fn catch_an_error() {
    test_pairs! {
    "(defun catch-an-error-error () \
       (error (quote some-error) \
       (quote error-description)))" => "[function catch-an-error-error]";
    "(catch-an-error-error)" => "some-error: error-description";
    "(catch-error (throw (catch-an-error-error)) \
       e \
       (quote caught-an-error))" => "caught-an-error";
    }
}
