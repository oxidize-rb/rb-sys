//! Allocator which reports mem usage to the Ruby GC
//!
//! This crate exports, one type, `RbAllocator`, which implements the `GlobalAlloc` trait. It is a
//! simple wrapper over the system allocator which reports memory usage to Ruby using
//! `rb_gc_adjust_memory_usage`

use crate::rb_gc_adjust_memory_usage;

type DiffSize = isize;

use std::alloc::{GlobalAlloc, Layout, System};

pub struct RbAllocator;

unsafe impl GlobalAlloc for RbAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            rb_gc_adjust_memory_usage(layout.size() as DiffSize)
        }
        ret
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc_zeroed(layout);
        if !ret.is_null() {
            rb_gc_adjust_memory_usage(layout.size() as DiffSize)
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        rb_gc_adjust_memory_usage(-(layout.size() as DiffSize));
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ret = System.realloc(ptr, layout, new_size);
        if !ret.is_null() {
            rb_gc_adjust_memory_usage(new_size as DiffSize - layout.size() as DiffSize);
        }
        ret
    }
}

#[macro_export]
macro_rules! ruby_global_allocator {
    () => {
        #[global_allocator]
        static RUBY_GLOBAL_ALLOCATOR: $crate::RbAllocator = $crate::RbAllocator;
    };
}
