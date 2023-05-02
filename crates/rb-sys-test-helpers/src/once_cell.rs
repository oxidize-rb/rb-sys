use std::sync::atomic::Ordering;

/// A simple implementation of a once cell so we can support Rust 2018. We
/// should drop this once we can since it's wasteful due to the spin-lock.
#[derive(Debug)]
pub struct OnceCell<T> {
    ready: std::sync::atomic::AtomicBool,
    value: std::sync::Once,
    value_ptr: std::cell::UnsafeCell<Option<T>>,
}

impl<T> OnceCell<T> {
    pub const fn new() -> Self {
        Self {
            ready: std::sync::atomic::AtomicBool::new(false),
            value: std::sync::Once::new(),
            value_ptr: std::cell::UnsafeCell::new(None),
        }
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.value.call_once(|| {
            let value = f();
            unsafe {
                *self.value_ptr.get() = Some(value);
                self.ready.store(true, Ordering::SeqCst);
            }
        });

        while !self.ready.load(Ordering::Acquire) && !self.value_ptr.get().is_null() {
            std::thread::yield_now();
        }

        unsafe { self.value_ptr.get().as_ref().unwrap().as_ref().unwrap() }
    }
}
