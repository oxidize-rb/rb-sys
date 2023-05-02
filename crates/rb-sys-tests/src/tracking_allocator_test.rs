use std::alloc::GlobalAlloc;

use rb_sys::tracking_allocator::{ManuallyTracked, TrackingAllocator};
use rb_sys_test_helpers::{capture_gc_stat_for, ruby_test};
use rusty_fork::rusty_fork_test;

rusty_fork_test! {
  #[ruby_test]
  fn test_tracking_allocator_works() {
    rb_sys::set_global_tracking_allocator!();

    let (my_vec, malloc_increase) = capture_gc_stat_for!("malloc_increase_bytes", {
      vec![1u32; 42]
    });

    assert_eq!(42 * 4, malloc_increase);

    let (_, malloc_increase_after_drop) = capture_gc_stat_for!("malloc_increase_bytes", {
      std::mem::drop(my_vec);
    });

    assert_eq!(-(42 * 4), malloc_increase_after_drop)
  }
}

#[ruby_test]
fn test_alloc_zeroed() {
    let allocator = TrackingAllocator::default();
    let layout = std::alloc::Layout::new::<[u8; 8]>();

    let zeroed_memory = unsafe { allocator.alloc_zeroed(layout) };
    let zeroed_slice = unsafe { std::slice::from_raw_parts(zeroed_memory, 8) };

    assert_eq!(zeroed_slice, &[0; 8]);

    unsafe { allocator.dealloc(zeroed_memory, layout) };
}

#[ruby_test]
fn test_alloc() {
    let allocator = TrackingAllocator::default();
    let layout = std::alloc::Layout::new::<[u8; 8]>();
    let memory = unsafe { allocator.alloc(layout) };
    let slice = unsafe { std::slice::from_raw_parts(memory, 8) };

    assert_eq!(8, slice.len());

    unsafe { allocator.dealloc(memory, layout) };
}

#[ruby_test]
fn test_realloc() {
    let allocator = TrackingAllocator::default();
    let layout = std::alloc::Layout::new::<[u8; 8]>();
    let memory_to_realloc = unsafe { allocator.alloc(layout) };
    let realloced_memory = unsafe { allocator.realloc(memory_to_realloc, layout, 16) };
    let realloced_slice = unsafe { std::slice::from_raw_parts(realloced_memory, 16) };

    assert_eq!(16, realloced_slice.len());

    unsafe { allocator.dealloc(realloced_memory, layout) };
}

#[ruby_test]
fn test_manually_tracked_reports_memory_usage_on_create() {
    let (_, increased) =
        capture_gc_stat_for!("malloc_increase_bytes", { ManuallyTracked::new((), 1024) });

    assert_eq!(1024, increased);
}

#[ruby_test]
fn test_manually_tracked_reports_memory_usage_on_drop() {
    let manually_tracked = ManuallyTracked::new((), 1024);

    let (_, decreased) = capture_gc_stat_for!("malloc_increase_bytes", {
        std::mem::drop(manually_tracked);
    });

    assert_eq!(-1024, decreased)
}

rusty_fork_test! {
  #[test]
  fn test_manually_tracked_works_without_ruby_vm_available() {
    let manually_tracked = ManuallyTracked::new(vec![1, 2, 3], 1024);

    assert_eq!(3, manually_tracked.len());

    std::mem::drop(manually_tracked);
  }
}
