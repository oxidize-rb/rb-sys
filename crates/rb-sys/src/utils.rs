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
#[allow(dead_code)]
pub(crate) unsafe fn is_ruby_vm_started() -> bool {
    #[cfg(ruby_engine = "mri")]
    let ret = {
        #[cfg(all(ruby_gt_2_4, ruby_lte_3_2))]
        let ret = !crate::hidden::ruby_current_vm_ptr.is_null();

        #[cfg(any(ruby_lte_2_4, ruby_gt_3_2))]
        let ret = crate::rb_cBasicObject != 0;

        ret
    };

    #[cfg(ruby_engine = "truffleruby")]
    let ret = crate::rb_cBasicObject != 0;

    ret
}

/// Macro for conditionally asserting type checks in Ruby, only active when RUBY_DEBUG is enabled.
/// This matches Ruby's behavior of only checking types in debug mode.
#[macro_export]
macro_rules! debug_ruby_assert_type {
    ($obj:expr, $type:expr, $message:expr) => {
        #[cfg(ruby_ruby_debug = "true")]
        {
            #[allow(clippy::macro_metavars_in_unsafe)]
            unsafe {
                assert!(
                    !$crate::SPECIAL_CONST_P($obj) && $crate::RB_BUILTIN_TYPE($obj) == $type,
                    $message
                );
            }
        }
        #[cfg(not(ruby_ruby_debug = "true"))]
        {
            let _ = ($obj, $type, $message); // Prevent unused variable warnings
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_fork::rusty_fork_test;
    use std::ptr::addr_of_mut;

    rusty_fork_test! {
        #[test]
        fn test_is_ruby_vm_started() {
            assert!(!unsafe { is_ruby_vm_started() });

            // Call ruby_sysinit which handles platform-specific initialization
            // (rb_w32_sysinit on Windows) and sets up standard file descriptors
            let mut argc = 0;
            let mut argv: [*mut std::os::raw::c_char; 0] = [];
            let mut argv_ptr = argv.as_mut_ptr();
            unsafe { crate::ruby_sysinit(&mut argc, &mut argv_ptr) };

            // ruby_init_stack must be called before ruby_setup, especially on
            // Windows where it's required for proper GC stack scanning
            let mut stack_marker: crate::VALUE = 0;
            unsafe { crate::ruby_init_stack(addr_of_mut!(stack_marker) as *mut _) };

            match unsafe { crate::ruby_setup() } {
                0 => {}
                code => panic!("Failed to setup Ruby (error code: {})", code),
            };

            assert!(unsafe { is_ruby_vm_started() });
        }
    }
}
