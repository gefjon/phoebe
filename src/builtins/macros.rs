macro_rules! special_form {
    ($name:expr ; ($($arg:tt)*) -> $blk:block) => {
        {
            use $crate::types::function::Function;
            use $crate::allocate::Allocate;
            
            let name = $crate::symbol_lookup::make_symbol(
                ::std::convert::AsRef::<[u8]>::as_ref($name)
            );
            make_arg_syms!($($arg)*);
            let body = Box::new(move || {
                get_args!($($arg)*);
                $blk
            });
            let arglist = make_arglist!($($arg)*);
            let func = Function::allocate(
                Function::make_special_form(
                    name,
                    arglist,
                    Box::leak(body)
                ).unwrap()
            );
            $crate::symbol_lookup::add_to_global(name, func);
        }
    };
}

macro_rules! builtin_func {
    ($name:expr ; ($($arg:tt)*) -> $blk:block) => {
        {
            use $crate::types::function::Function;
            use $crate::allocate::Allocate;
            
            let name = $crate::symbol_lookup::make_symbol(
                ::std::convert::AsRef::<[u8]>::as_ref($name)
            );
            make_arg_syms!($($arg)*);
            let body = Box::new(move || {
                get_args!($($arg)*);
                $blk
            });
            let arglist = make_arglist!($($arg)*);
            let func = Function::allocate(
                Function::make_builtin(
                    name,
                    arglist,
                    Box::leak(body)
                ).unwrap()
            );
            $crate::symbol_lookup::add_to_global(name, func);
        }
    };
}

macro_rules! make_arg_syms {
    ($($arg:ident)*) => {
        $(let $arg = $crate::symbol_lookup::make_symbol(stringify!($arg).as_ref());)*;
    };
    ($($arg:ident)* &optional $($oarg:ident)*) => {
        $(let $arg = $crate::symbol_lookup::make_symbol(stringify!($arg).as_ref());)*;
        $(let $oarg = $crate::symbol_lookup::make_symbol(stringify!($oarg).as_ref());)*;
    };
    ($($arg:ident)* &rest $($rarg:ident)*) => {
        $(let $arg = $crate::symbol_lookup::make_symbol(stringify!($arg).as_ref());)*;
        $(let $rarg = $crate::symbol_lookup::make_symbol(stringify!($rarg).as_ref());)*;
    };
    ($($arg:ident)* &optional $($oarg:ident)* &rest $($rarg:ident)*) => {
        $(let $arg = $crate::symbol_lookup::make_symbol(stringify!($arg).as_ref());)*;
        $(let $oarg = $crate::symbol_lookup::make_symbol(stringify!($oarg).as_ref());)*;
        $(let $rarg = $crate::symbol_lookup::make_symbol(stringify!($rarg).as_ref());)*;
    };
}

macro_rules! get_args {
    ($($arg:ident)*) => {
        $(let $arg = $crate::symbol_lookup::lookup_symbol($arg);)*;
    };
    ($($arg:ident)* &optional $($oarg:ident)*) => {
        $(let $arg = $crate::symbol_lookup::lookup_symbol($arg);)*;
        $(let $oarg = $crate::symbol_lookup::lookup_symbol($oarg);)*;
    };
    ($($arg:ident)* &rest $($rarg:ident)*) => {
        $(let $arg = $crate::symbol_lookup::lookup_symbol($arg);)*;
        $(let $rarg = $crate::symbol_lookup::lookup_symbol($rarg);)*;
    };
    ($($arg:ident)* &optional $($oarg:ident)* &rest $($rarg:ident)*) => {
        $(let $arg = $crate::symbol_lookup::lookup_symbol($arg);)*;
        $(let $oarg = $crate::symbol_lookup::lookup_symbol($oarg);)*;
        $(let $rarg = $crate::symbol_lookup::lookup_symbol($rarg);)*;
    };
}

macro_rules! make_arglist {
    ($($arg:ident)*) => {
        {
            let mut arglist = $crate::types::list::List::nil();
            $(arglist = arglist.push($crate::types::Object::from($arg));)*;
            unsafe { arglist.nreverse() }
        };
    };
    ($($arg:ident)* &optional $($oarg:ident)*) => {
        {
            let mut arglist = $crate::types::list::List::nil();
            $(arglist = arglist.push($crate::types::Object::from($arg));)*;
            arglist = arglist.push(
                Object::from(*$crate::types::function::OPTIONAL)
            );
            $(arglist = arglist.push($crate::types::Object::from($oarg));)*;
            unsafe { arglist.nreverse() }
        }
    };
    ($($arg:ident)* &rest $($rarg:ident)*) => {
        {
            let mut arglist = $crate::types::list::List::nil();
            $(arglist = arglist.push($crate::types::Object::from($arg));)*;
            arglist = arglist.push(
                Object::from(*$crate::types::function::REST)
            );
            $(arglist = arglist.push($crate::types::Object::from($rarg));)*;
            unsafe { arglist.nreverse() }
        }
    };
    ($($arg:ident)* &optional $($oarg:ident)* &rest $($rarg:ident)*) => {
        {
            let mut arglist = $crate::types::list::List::nil();
            $(arglist = arglist.push($crate::types::Object::from($arg));)*;
            arglist = arglist.push(
                Object::from(*$crate::types::function::OPTIONAL)
            );
            $(arglist = arglist.push($crate::types::Object::from($oarg));)*;
            arglist = arglist.push(
                Object::from(*$crate::types::function::REST)
            );
            $(arglist = arglist.push($crate::types::Object::from($rarg));)*;
            unsafe { arglist.nreverse() }
        }
    };
}

#[macro_export]
macro_rules! builtin_functions {
    ($($name:tt ($($arg:tt)*) -> $blk:block);* $(;)*) => {{
        $(builtin_func!($name ; ($($arg)*) -> $blk);)*;
    }};
}

#[macro_export]
macro_rules! special_forms {
    ($($name:tt ($($arg:tt)*) -> $blk:block);* $(;)*) => {{
        $(special_form!($name ; ($($arg)*) -> $blk);)*;
    }};
}