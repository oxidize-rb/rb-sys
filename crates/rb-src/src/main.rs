use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut rb = rb_src::Build::default();

    rb.ruby_version("3.2.0")
        .prefix("/tmp/ruby")
        .build_dir("/tmp/ruby-build")
        .build()?;

    Ok(())
}
