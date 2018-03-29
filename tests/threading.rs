extern crate phoebe;
use std::thread;

fn test_input_output_pairs(pairs: &[(&[u8], &[u8])]) {
    use phoebe::repl::{initialize, read_eval_print_loop};
    use std::str;

    initialize();
    for &(input, output) in pairs {
        let mut input_buf = input.to_vec();
        let mut output_buf = Vec::with_capacity(output.len());
        let mut error_buf = Vec::new();
        let mut inp = &input_buf[..];

        read_eval_print_loop(&mut inp, &mut output_buf, &mut error_buf, false).unwrap();
        if !error_buf.is_empty() {
            panic!(
                "read_eval_print_loop errored: {}",
                str::from_utf8(&error_buf).unwrap()
            );
        }
        assert_eq!(
            str::from_utf8(output),
            str::from_utf8(&output_buf),
            "{} returned {}",
            str::from_utf8(&input_buf).unwrap(),
            str::from_utf8(&output_buf).unwrap()
        );
    }
}

#[test]
fn do_two_things_in_different_threads() {
    let first_child = thread::spawn(move || {
        test_input_output_pairs(&[
            (b"(defvar x 5)", b"5\n"),
            (b"(defun return-x () x)", b"[function return-x]\n"),
            (b"(list (return-x) x)", b"(5 5)\n"),
        ]);
    });
    let second_child = thread::spawn(move || {
        test_input_output_pairs(&[
            (b"(defvar y 2)", b"2\n"),
            (b"(defun return-y () y)", b"[function return-y]\n"),
            (b"(list (return-y) y)", b"(2 2)\n"),
        ]);
    });
    first_child.join().expect("Thread first_child paniced!");
    second_child.join().expect("Thread second_child paniced!");
}
