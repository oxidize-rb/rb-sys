//! Support for reporting Rust memory usage to the Ruby GC.

use crate::{rb_gc_adjust_memory_usage, utils::is_ruby_vm_available};
use std::alloc::{GlobalAlloc, Layout, System};

/// A simple wrapper over [`std::alloc::System`] which reports memory usage to
/// the Ruby GC. This gives the GC a more accurate picture of the process'
/// memory usage so it can make better decisions about when to run.
///
/// # Example
/// ```
/// use rb_sys::TrackingAllocator;
///
/// let mut vec = Vec::new_in(TrackingAllocator::default());
/// ```
#[derive(Debug, Default)]
pub struct TrackingAllocator;

impl TrackingAllocator {
    /// Adjust the memory usage reported to the Ruby GC by `delta`. Useful for
    /// tracking allocations invisible to the Rust allocator, such as `mmap` or
    /// direct `malloc` calls.
    ///
    /// # Example
    /// ```
    /// use rb_sys::TrackingAllocator;
    ///
    /// // Allocate 1024 bytes of memory using `mmap` or `malloc`...
    /// TrackingAllocator::adjust_memory_usage(1024);
    ///
    /// // ...and then after the memory is freed, adjust the memory usage again.
    /// TrackingAllocator::adjust_memory_usage(-1024);
    /// ```
    pub fn adjust_memory_usage(delta: isize) {
        #[cfg(target_pointer_width = "32")]
        let delta = delta as i32;

        #[cfg(target_pointer_width = "64")]
        let delta = delta as i64;

        unsafe {
            if is_ruby_vm_available() {
                rb_gc_adjust_memory_usage(delta);
            }
        }
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        let delta = layout.size() as isize;

        if !ret.is_null() && delta != 0 {
            Self::adjust_memory_usage(delta)
        }

        ret
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc_zeroed(layout);
        let delta = layout.size() as isize;

        if !ret.is_null() && delta != 0 {
            Self::adjust_memory_usage(delta)
        }

        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        let delta = -(layout.size() as isize);

        if delta != 0 {
            Self::adjust_memory_usage(delta);
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ret = System.realloc(ptr, layout, new_size);
        let delta = new_size as isize - layout.size() as isize;

        if !ret.is_null() && delta != 0 {
            Self::adjust_memory_usage(delta);
        }

        ret
    }
}

/// Set the global allocator to [`TrackingAllocator`].
///
/// # Example
/// ```
/// // File: ext/my_gem/src/lib.rs
/// use rb_sys::set_global_tracking_allocator;
///
/// set_global_tracking_allocator!();
/// ```
#[macro_export]
macro_rules! set_global_tracking_allocator {
    () => {
        #[global_allocator]
        static RUBY_GLOBAL_TRACKING_ALLOCATOR: $crate::tracking_allocator::TrackingAllocator =
            $crate::tracking_allocator::TrackingAllocator;
    };
}

