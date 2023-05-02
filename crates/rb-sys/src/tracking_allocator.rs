//! Support for reporting Rust memory usage to the Ruby GC.

use crate::{rb_gc_adjust_memory_usage, utils::is_ruby_vm_available};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicIsize, Ordering},
};

/// A simple wrapper over [`std::alloc::System`] which reports memory usage to
/// the Ruby GC. This gives the GC a more accurate picture of the process'
/// memory usage so it can make better decisions about when to run.
#[derive(Debug)]
pub struct TrackingAllocator;

impl TrackingAllocator {
    /// Create a new [`TrackingAllocator`].
    pub const fn new() -> Self {
        Self
    }

    /// Create a new [`TrackingAllocator`] with default values.
    pub const fn default() -> Self {
        Self::new()
    }

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
    pub fn adjust_memory_usage(delta: isize) -> isize {
        if delta == 0 {
            return 0;
        }

        #[cfg(target_pointer_width = "32")]
        let delta = delta as i32;

        #[cfg(target_pointer_width = "64")]
        let delta = delta as i64;

        unsafe {
            if is_ruby_vm_available() {
                rb_gc_adjust_memory_usage(delta);
                delta as isize
            } else {
                0
            }
        }
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        let delta = layout.size() as isize;

        if !ret.is_null() && delta != 0 {
            Self::adjust_memory_usage(delta);
        }

        ret
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc_zeroed(layout);
        let delta = layout.size() as isize;

        if !ret.is_null() && delta != 0 {
            Self::adjust_memory_usage(delta);
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

/// A guard which adjusts the memory usage reported to the Ruby GC by `delta`.
/// This allows you to track resources which are invisible to the Rust
/// allocator, such as items that are known to internally use `mmap` or direct
/// `malloc` in their implementation.
///
/// # Example
/// ```
/// use rb_sys::tracking_allocator::ManuallyTracked;
///
/// type SomethingThatUsedMmap = ();
///
/// // Will tell the Ruby GC that 1024 bytes were allocated.
/// let item = ManuallyTracked::new(SomethingThatUsedMmap, 1024);
///
/// // Will tell the Ruby GC that 1024 bytes were freed.
/// std::mem::drop(item);
/// ```
#[derive(Debug)]
pub struct ManuallyTracked<T> {
    item: T,
    memsize_delta: AtomicIsize,
}

impl<T> ManuallyTracked<T> {
    /// Create a new `ManuallyTracked<T>`, and immediately report that `memsize`
    /// bytes were allocated.
    pub fn wrap(item: T, memsize: usize) -> Self {
        let delta = TrackingAllocator::adjust_memory_usage(memsize as isize);

        Self {
            item,
            memsize_delta: AtomicIsize::new(delta),
        }
    }

    /// Increase the memory usage reported to the Ruby GC by `memsize` bytes.
    pub fn increase_memory_usage(&self, memsize: usize) {
        self.memsize_delta
            .fetch_add(memsize as isize, Ordering::SeqCst);
        TrackingAllocator::adjust_memory_usage(memsize as isize);
    }

    /// Decrease the memory usage reported to the Ruby GC by `memsize` bytes.
    pub fn decrease_memory_usage(&self, memsize: usize) {
        self.memsize_delta
            .fetch_sub(memsize as isize, Ordering::SeqCst);

        TrackingAllocator::adjust_memory_usage(-(memsize as isize));
    }

    /// Get the current memory usage delta.
    pub fn memsize_delta(&self) -> isize {
        self.memsize_delta.load(Ordering::SeqCst)
    }

    /// Get a shared reference to the inner `T`.
    pub fn get(&self) -> &T {
        &self.item
    }

    /// Get a mutable reference to the inner `T`.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl Default for ManuallyTracked<()> {
    fn default() -> Self {
        Self::wrap((), 0)
    }
}

impl<T> Drop for ManuallyTracked<T> {
    fn drop(&mut self) {
        let memsize = self.memsize_delta.swap(0, Ordering::SeqCst);
        TrackingAllocator::adjust_memory_usage(0 - memsize);
    }
}
