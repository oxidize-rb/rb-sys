use rb_sys::tracking_allocator::{ManuallyTracked, TrackingAllocator};
use rb_sys_test_helpers::{capture_gc_stat_for, ruby_test};
use rusty_fork::rusty_fork_test;
use std::alloc::GlobalAlloc;

rusty_fork_test! {
  #[ruby_test]
  fn test_tracking_allocator_works() {
    rb_sys::set_global_tracking_allocator!();

    let (my_vec, malloc_increase) = capture_gc_stat_for!("malloc_increase_bytes", {
      vec![1u32; 42]
    });

    assert_eq!(42 * 4, malloc_increase);
    assert_eq!(42, my_vec.len());
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
        capture_gc_stat_for!("malloc_increase_bytes", { ManuallyTracked::wrap((), 1024) });

    assert_eq!(1024, increased);
}

#[ruby_test]
fn test_manually_tracked_reports_memory_usage_on_drop() {
    let manually_tracked = ManuallyTracked::wrap((), 1024);

    let (_, decreased) = capture_gc_stat_for!("malloc_increase_bytes", {
        std::mem::drop(manually_tracked);
    });

    assert_eq!(-1024, decreased)
}

#[ruby_test]
fn test_manually_tracked_default() {
    let manually_tracked = ManuallyTracked::default();

    assert_eq!(&(), manually_tracked.get());

    let (_, increased) = capture_gc_stat_for!("malloc_increase_bytes", {
        manually_tracked.increase_memory_usage(1024);
    });

    assert_eq!(1024, increased);

    manually_tracked.decrease_memory_usage(1024);

    let (_, decreased) = capture_gc_stat_for!("malloc_increase_bytes", {
        std::mem::drop(manually_tracked);
    });

    assert_eq!(0, decreased);
}

#[ruby_test]
fn test_manually_tracked_allows_for_increasing_and_decreasing() {
    let manually_tracked = ManuallyTracked::wrap((), 0);

    let (_, changed) = capture_gc_stat_for!("malloc_increase_bytes", {
        manually_tracked.increase_memory_usage(1024);
        manually_tracked.decrease_memory_usage(256);
    });

    assert_eq!(768, changed);
}

#[ruby_test]
fn test_manually_tracked_decreases_on_drop() {
    let manually_tracked = ManuallyTracked::wrap((), 1024);

    let (_, decreased) = capture_gc_stat_for!("malloc_increase_bytes", {
        std::mem::drop(manually_tracked);
    });

    assert_eq!(-1024, decreased);
}

#[ruby_test]
fn test_adjusting_with_many_threads_works() {
    let manually_tracked = ManuallyTracked::wrap((), 0);
    let mut threads = Vec::new();

    let (_, _reported_bytes) = capture_gc_stat_for!("malloc_increase_bytes", {
        let manually_tracked_ptr = &manually_tracked as *const ManuallyTracked<()>;
        let tracked = unsafe { &*manually_tracked_ptr }; // no mutexes here :D

        for _ in 0..10 {
            threads.push(std::thread::spawn(move || {
                for i in 0..1000 {
                    if i % 2 == 0 {
                        tracked.increase_memory_usage(1);
                    } else {
                        tracked.decrease_memory_usage(1);
                    }
                }
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }
    });

    let delta = manually_tracked.memsize_delta();
    assert_eq!(0, delta);
    std::mem::drop(manually_tracked);

    // Ideally, we'd test this too, but it seems the reported bytes are not
    // actually atomic... So, just best effort here.
    assert_eq!(0, _reported_bytes);
}

rusty_fork_test! {
  #[test]
  fn test_manually_tracked_works_without_ruby_vm_available() {
    let manually_tracked = ManuallyTracked::wrap(vec![1, 2, 3], 1024);

    assert_eq!(3, manually_tracked.get().len());

    std::mem::drop(manually_tracked);
  }
}
