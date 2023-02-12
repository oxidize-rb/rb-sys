use crate::rb_config::RbConfig;

pub struct Build;

/// A wrapper around `cc::Build` that sets up a build with the proper flags for
/// compiling C code that links to Ruby. This can be useful for compiling macros
/// or other C code that is only accessible from C.
impl Build {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> cc_impl::Build {
        let rb = RbConfig::current();
        let mut build = cc_impl::Build::new();
        let cc_args = rb.get("CC");
        let mut cc_args = cc_args.split_whitespace().collect::<Vec<_>>();

        cc_args.reverse();
        build.compiler(cc_args.pop().expect("CC is empty"));
        cc_args.reverse();

        for arg in cc_args {
            build.flag(arg);
        }

        let rubyhdrdir = rb.get("rubyhdrdir");
        build.include(&rubyhdrdir);
        build.include(format!("{}/include/internal", &rubyhdrdir));
        build.include(format!("{}/include/impl", &rubyhdrdir));
        build.include(rb.get("rubyarchhdrdir"));

        for flag in &rb.cflags() {
            build.flag_if_supported(flag);
        }

        build.flag_if_supported("-Wno-unused-parameter");

        build
    }
}
