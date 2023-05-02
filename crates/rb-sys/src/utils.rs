//! Internal utility function.

/// Check if the Ruby VM is available on the current thread.
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
/// simple way to check if the VM is available on the current thread. So we just
/// assume it is for now. See https://bugs.ruby-lang.org/issues/19627.
pub(crate) unsafe fn is_ruby_vm_available() -> bool {
    #[cfg(all(ruby_gt_2_4, ruby_lte_3_2))]
    let ret = !crate::hidden::ruby_current_vm_ptr.is_null();

    #[cfg(any(ruby_lte_2_4, ruby_gt_3_2))]
    let ret = crate::rb_cBasicObject != 0;

    ret
}
