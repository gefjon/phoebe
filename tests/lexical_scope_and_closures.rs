extern crate phoebe;

fn test_input_output_pairs(pairs: &[(&[u8], &[u8])]) {
    use phoebe::repl::{initialize, read_eval_print_loop};
    use std::str;

    initialize();
    for &(input, output) in pairs {
        let mut input_buf = input.to_vec();
        let mut output_buf = Vec::with_capacity(output.len());
        let mut error_buf = Vec::new();
        let mut inp = &input_buf[..];

        println!("inp is {}", str::from_utf8(inp).unwrap());

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
        println!("got output {}", str::from_utf8(&output_buf).unwrap());
    }
}

#[test]
fn define_and_call_a_closure() {
    test_input_output_pairs(&[
        (b"(defvar x 5)", b"5\n"),
        (b"(defun returns-five () x)", b"[function returns-five]\n"),
        (
            b"(let ((x 3)) (defun returns-three () x))",
            b"[function returns-three]\n",
        ),
        (b"(returns-five)", b"5\n"),
        (b"(returns-three)", b"3\n"),
    ]);
}
