#[macro_use]
extern crate phoebe;

use phoebe::repl::test_utilities::test_input_output_pairs;

#[test]
fn cond_lexical_scoping() {
    test_pairs! {
        "(defvar cond-lexical-scoping-param 5)" => "5";
        "(let ((cond-lexical-scoping-param 3)) \
           (cond (t cond-lexical-scoping-param)))" => "3";
    }
}

#[test]
fn let_lexical_scoping() {
    test_pairs! {
        "(defvar let-lexical-scoping-param 0)" => "0";
        "(defun let-lexical-scoping-test (let-lexical-scoping-param) \
         (let ((foo let-lexical-scoping-param)) foo))" => "[function let-lexical-scoping-test]";
        "(let-lexical-scoping-test 5)" => "5";
    }
}

#[test]
fn setf_lexical_scoping() {
    test_pairs! {
        "(defvar setf-lexical-scoping-input 0)" => "0";
        "(defvar setf-lexical-scoping-output 1)" => "1";
        "(let ((setf-lexical-scoping-input 2)) \
         (setf setf-lexical-scoping-output setf-lexical-scoping-input))" => "2";
        "setf-lexical-scoping-output" => "2";
        "setf-lexical-scoping-input" => "0";
    }
}

#[test]
fn defvar_lexical_scoping() {
    test_pairs! {
        "(defvar defvar-lexical-scoping-param 5)" => "5";
        "(let ((defvar-lexical-scoping-param 3)) \
           (defvar defvar-lexical-scoping-param-2 3))" => "3";
        "defvar-lexical-scoping-param-2" => "3";
    }
}
