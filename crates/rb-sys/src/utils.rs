//! Internal utility functions.

/// Check if the Ruby VM is globally available.
///
/// Unfortunately there is no public API for this check, but there's a hidden
/// `ruby_current_vm_ptr` symbol in libruby 2.5 - 3.2 which we can use to
/// determine if the VM has been initialized, or shut down.
///
/// # Notes
///
/// Ruby 2.4 and below don't have a global VM pointer, so we can't check if it's
/// null. Ruby 2.4 is EOL, and support will be dropped soon anyway.
//
/// In Ruby 3.3, the global VM pointer is no longer exported, so there's no
/// simple way to check the global VM pointer, so instead we check if known
/// static value is non-zero.
///
/// On Ruby < 3.3, we also need to check if the global VM pointer is null to
/// ensure the VM hasn't stopped, which makes the function name a bit of a
/// misnomer... but in actuality this function can only guarantee that the
/// VM is started, not that it's still running.
pub(crate) unsafe fn is_ruby_vm_started() -> bool {
    #[cfg(all(ruby_gt_2_4, ruby_lte_3_2))]
    let ret = !crate::hidden::ruby_current_vm_ptr.is_null();

    #[cfg(any(ruby_lte_2_4, ruby_gt_3_2))]
    let ret = crate::rb_cBasicObject != 0;

    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use rb_sys_test_helpers::with_ruby_vm;

    #[test]
    fn test_is_ruby_vm_started() {
        assert!(!unsafe { is_ruby_vm_started() });

        with_ruby_vm(|| {
            assert!(unsafe { is_ruby_vm_started() });
        })
        .unwrap();
    }
}
