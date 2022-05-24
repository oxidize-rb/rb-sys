rb_sys::ruby_extension!();

#[cfg(unix)]
#[cfg(ruby_gte_3_2 = "true")]
#[test]
fn test_ruby_abi_version() {
    assert!(ruby_abi_version() == 1)
}

#[cfg(unix)]
#[cfg(ruby_gte_3_2 = "false")]
#[test]
fn test_ruby_abi_version() {
    assert!(ruby_abi_version() == 0)
}
