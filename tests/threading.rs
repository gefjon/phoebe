extern crate phoebe;

use phoebe::repl::test_utilities::test_input_output_pairs;
use std::thread;

#[test]
fn do_two_things_in_different_threads() {
    let first_child = thread::spawn(move || {
        test_input_output_pairs(&[
            ("(defvar x 5)", "5\n"),
            ("(defun return-x () x)", "[function return-x]\n"),
            ("(list (return-x) x)", "(5 5)\n"),
        ])
        .unwrap();
    });
    let second_child = thread::spawn(move || {
        test_input_output_pairs(&[
            ("(defvar y 2)", "2\n"),
            ("(defun return-y () y)", "[function return-y]\n"),
            ("(list (return-y) y)", "(2 2)\n"),
        ])
        .unwrap();
    });
    first_child.join().expect("Thread first_child paniced!");
    second_child.join().expect("Thread second_child paniced!");
}

#[test]
fn many_threads_at_once() {
    use std::thread::{spawn, JoinHandle};
    const NUMBER_OF_THREADS: usize = 32;
    fn thread_inner(sym: usize) {
        test_input_output_pairs(&[
            (
                &format!("(defvar make-a-thread-{0} {0})", sym),
                &format!("{}\n", sym),
            ),
            (
                &format!(
                    "(defun make-a-thread-fn-{0} () \
                     (* make-a-thread-{0} {0}))",
                    sym
                ),
                &format!("[function make-a-thread-fn-{}]\n", sym),
            ),
            (
                &format!("(setf make-a-thread-{0} (make-a-thread-fn-{0}))", sym),
                &format!("{}\n", sym * sym),
            ),
            (
                &format!("make-a-thread-{}", sym),
                &format!("{}\n", sym * sym),
            ),
        ])
        .unwrap();
    }
    fn make_a_thread(sym: usize) -> JoinHandle<()> {
        spawn(move || thread_inner(sym))
    }

    let mut handles = Vec::with_capacity(NUMBER_OF_THREADS);

    for i in 0..NUMBER_OF_THREADS {
        handles.push(make_a_thread(i));
    }

    for handle in handles.drain(..) {
        handle.join().expect("A thread errored");
    }
}
