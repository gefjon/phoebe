[![Build Status](https://travis-ci.org/gefjon/phoebe.svg?branch=master)](https://travis-ci.org/gefjon/phoebe)


# Phoebe - a simple Lisp

Phoebe is a from-the-ground rebuild of [Rlisp](https://github.com/gefjon/rlisp),
a first attempt at a Lisp interpreter whose low code quality quickly got out of
hand.

You can run Phoebe with `cargo run --bin repl`. Syntax documentation is coming,
but the language is currently in its very early stages and liable to change at
any time.

## TODO:
- [ ] Write more extensive tests, particularly of the garbage collector.
- [ ] Implement built-in functions and special forms
  + Special forms:
    * [x] `cond`
    * [x] `let`
    * [x] `lambda`
    * [ ] `if`
    * [ ] `when`
    * [ ] `unless`
    * [ ] `defun`
    * [ ] `defvar`
    * [ ] `setf`
    * [ ] `nref`
    * [ ] `in-namespace`
    * [ ] ...and many more!
  + Built-in functions
    * [x] `list`
    * [ ] `cons`
    * [ ] `error`
    * [ ] `make-namespace`
    * [ ] `use-namespace`
    * [ ] `+`
    * [ ] `-`
    * [ ] `*`
    * [ ] `/`
    * [ ] `mod`
    * [ ] `rem`
    * [ ] `floor`
    * [ ] `ceil`
    * [ ] `trunc`
    * [ ] `round`
    * [ ] `assert`
    * [ ] `and`
    * [ ] `or`
    * [ ] `xor`
    * [ ] ...and many more!
- [ ] Write tests for built-in functions and special forms
  + Special forms:
    * [ ] `cond`
    * [ ] `let`
    * [ ] `lambda`
  + Built-in functions:
    * [x] `list`
- [ ] Strings
- [ ] Arrays
- [ ] I/O?
- [ ] Threading?
- [ ] Byte-compilation?
- [ ] FFI?
- [ ] Machine-code compilation?
