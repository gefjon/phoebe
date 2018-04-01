#[macro_use]
extern crate phoebe;

#[test]
fn define_and_call() {
    test_pairs! {
        "(defun define-and-call-fn (&key x y z) \
           (cond \
             (x x) \
             (y y) \
             (z z) \
             (t nil)))" => "[function define-and-call-fn]";
        "(define-and-call-fn :x 1 :y 2 :z 3)" => "1";
        "(define-and-call-fn :z 3 :y 2 :x 1)" => "1";
        "(define-and-call-fn :z 3)" => "3";
        "(define-and-call-fn :y 2 :z 3)" => "2";
    }
}
