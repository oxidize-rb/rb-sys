/// A macro which asserts the object is a given Ruby type.
#[macro_export]
macro_rules! debug_assert_ruby_type {
    ($obj:expr, $type:expr) => {
        debug_assert!($crate::RB_BUILTIN_TYPE($obj) == $type);
    };
}

/// A macro to assert a Ruby flag is set
#[macro_export]
macro_rules! refute_flag {
    ($flags:expr, $flag:expr) => {
        assert!(
            $flags & ($flag as $crate::VALUE) == 0,
            "{:?} flag was unexpectedly set",
            stringify!($flag)
        );
    };
}
