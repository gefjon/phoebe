#[macro_use]
extern crate phoebe;

#[test]
fn make_and_reference() {
    test_pairs! {
        "(make-namespace :name make-and-reference)"
            => "[namespace make-and-reference]";
        "(nref make-and-reference foo)" => "UNINITIALIZED";
        "(setf (nref make-and-reference foo) 3)" => "3";
        "(nref make-and-reference foo)" => "3";
    }
}

#[test]
fn with_contents() {
    test_pairs! {
        "(make-namespace :name with-contents \
           :contents ((one 1) \
                       (two 2) \
                       (three 3)))"
            => "[namespace with-contents]";
        "(nref with-contents one)" => "1";
        "(nref with-contents two)" => "2";
        "(setf (nref with-contents two) (nref with-contents one))"
            => "1";
        "(nref with-contents two)" => "1";
    }
}
