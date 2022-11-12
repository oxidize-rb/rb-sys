fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rb_env = rb_sys_env::activate()?;

    if rb_env.ruby_major_minor() >= (3, 1) && cfg!(windows) {
        println!("cargo:rustc-cfg=windows_broken_vm_init_3_1");
    }

    Ok(())
}
