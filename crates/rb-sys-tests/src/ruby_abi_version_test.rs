#[cfg(all(ruby_has_ruby_abi_version, unix))]
#[rb_sys_test_helpers::ruby_test]
fn test_ruby_abi_version() {
    assert!(ruby_abi_version() >= 1)
}

#[cfg(all(ruby_lt_3_2, unix))]
#[rb_sys_test_helpers::ruby_test]
fn test_ruby_abi_version() {
    assert_eq!(ruby_abi_version(), 0)
}
