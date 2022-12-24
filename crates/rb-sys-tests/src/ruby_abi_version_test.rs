rb_sys::ruby_abi_version!();

#[cfg(all(ruby_has_ruby_abi_version, unix))]
#[test]
fn test_ruby_abi_version() {
    assert!(ruby_abi_version() >= 1)
}

#[cfg(all(ruby_lt_3_2, unix))]
#[test]
fn test_ruby_abi_version() {
    assert_eq!(ruby_abi_version(), 0)
}
