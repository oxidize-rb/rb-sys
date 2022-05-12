#[cfg(unix)]
#[cfg(ruby_major = "3")]
#[cfg(ruby_minor = "2")]
rb_sys::ruby_extension!();

#[cfg(unix)]
#[cfg(ruby_major = "3")]
#[cfg(ruby_minor = "2")]
#[test]
fn test_ruby_abi_version() {
    assert!(ruby_abi_version() == 1)
}
