use crate::rb_config;

pub struct Build;

/// A wrapper around `cc::Build` that sets up a build with the proper flags for
/// compiling C code that links to Ruby. This can be useful for compiling macros
/// or other C code that is only accessible from C.
impl Build {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> cc_impl::Build {
        let rb = rb_config();
        let mut build = cc_impl::Build::new();
        let cc_args = rb.get("CC");
        let mut cc_args = cc_args.split_whitespace().collect::<Vec<_>>();

        cc_args.reverse();
        build.compiler(cc_args.pop().expect("CC is empty"));
        cc_args.reverse();

        for arg in cc_args {
            build.flag(arg);
        }

        build.include(rb.get("rubyhdrdir"));
        build.include(rb.get("rubyarchhdrdir"));
        build.include(format!("{}/include/internal", rb.get("rubyhdrdir")));
        build.include(format!("{}/include/impl", rb.get("rubyhdrdir")));

        for flag in &rb.cflags {
            build.flag(flag);
        }

        build
    }
}
