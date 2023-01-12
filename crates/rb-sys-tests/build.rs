use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = rb_sys_env::activate()?;

    Ok(())
}
