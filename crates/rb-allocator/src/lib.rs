//! Allocator which reports mem usage to the Ruby GC
//!
//! This crate exports, one type, `RbAllocator`, which implements the `GlobalAlloc` trait. It is a
//! simple wrapper over the system allocator which reports memory usage to Ruby using
//! `rb_gc_adjust_memory_usage`

use rb_sys::{rb_gc_adjust_memory_usage, ssize_t};
use std::alloc::{GlobalAlloc, Layout, System};

pub struct RbAllocator;

unsafe impl GlobalAlloc for RbAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            rb_gc_adjust_memory_usage(layout.size() as ssize_t)
        }
        ret
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc_zeroed(layout);
        if !ret.is_null() {
            rb_gc_adjust_memory_usage(layout.size() as ssize_t)
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        rb_gc_adjust_memory_usage(0 - (layout.size() as ssize_t));
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ret = System.realloc(ptr, layout, new_size);
        if !ret.is_null() {
            rb_gc_adjust_memory_usage((new_size as isize - layout.size() as isize) as ssize_t);
        }
        ret
    }
}

#[macro_export]
macro_rules! ruby_global_allocator {
    () => {
        use $crate::RbAllocator;
        #[global_allocator]
        static RUBY_GLOBAL_ALLOCATOR: RbAllocator = RbAllocator;
    };
}
