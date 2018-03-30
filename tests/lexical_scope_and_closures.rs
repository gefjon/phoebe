#[macro_use]
extern crate phoebe;

#[test]
fn define_and_call_a_closure() {
    test_pairs! {
        "(defvar test-param 5)" => "5";
        "(defun returns-five () test-param)" => "[function returns-five]";
        "(let ((test-param 3)) (defun returns-three () test-param))" => "[function returns-three]";
        "(returns-five)" => "5";
        "(returns-three)" => "3";
    }
}
