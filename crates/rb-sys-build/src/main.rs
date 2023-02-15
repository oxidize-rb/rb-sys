use std::io;

use rb_sys_build::{bindings::Builder, RbConfig};

fn main() {
    let rb = RbConfig::current();

    let builder = Builder::new()
        .docs(false)
        .rbimpls(true)
        .append_cflags(&rb.cflags())
        .append_cflags(&rb.cppflags())
        .include(rb.get("rubyhdrdir"))
        .include(rb.get("rubyarchhdrdir"))
        .deprecated_types(true);

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_code_to(&mut io::stdout())
        .expect("Unable to write bindings");
}
