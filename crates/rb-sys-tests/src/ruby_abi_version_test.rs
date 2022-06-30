rb_sys::ruby_abi_version!();

#[cfg(unix)]
#[cfg(ruby_version_gte_3_2 = "true")]
#[test]
fn test_ruby_abi_version() {
    assert_eq!(ruby_abi_version(), 1)
}

#[cfg(unix)]
#[cfg(ruby_version_gte_3_2 = "false")]
#[test]
fn test_ruby_abi_version() {
    assert_eq!(ruby_abi_version(), 0)
}
