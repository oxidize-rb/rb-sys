use std::error::Error;

/// Generate the bindings for current Ruby installation.
pub fn run() -> Result<(), Box<dyn Error>> {
    rb_sys_local::run();
    Ok(())
}
