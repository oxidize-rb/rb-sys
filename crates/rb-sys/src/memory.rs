/// Prevents premature destruction of local objects.
///
/// Ruby's garbage collector is conservative; it scans the C level machine stack as well.
/// Possible in-use Ruby objects must remain visible on stack, to be properly marked as such.
/// However, Rust's compiler optimizations might remove the references to these objects from
/// the stack when they are not being used directly.
///
/// Consider the following example:
///
/// ```ignore
/// use rb_sys::{rb_str_new_cstr, rb_str_cat_cstr, RSTRING_PTR, rb_gc_guard};
///
/// unsafe {
///     let s = rb_str_new_cstr(" world\0".as_ptr() as _);
///     let sptr = RSTRING_PTR(s);
///     let t = rb_str_new_cstr("hello,\0".as_ptr() as _); // Possible GC invocation
///     let u = rb_str_cat_cstr(t, sptr);
///     rb_gc_guard!(s); // ensure `s` (and thus `sptr`) do not get GC-ed
/// }
/// ```
///
/// In this example, without the `rb_gc_guard!`, the last use of `s` is before the last use
/// of `sptr`. Compilers could think `s` and `t` are allowed to overlap. That would
/// eliminate `s` from the stack, while `sptr` is still in use. If our GC runs at that
/// very moment, `s` gets swept out, which also destroys `sptr`.
///
/// In order to prevent this scenario, `rb_gc_guard!` must be placed after the last use
/// of `sptr`. Placing `rb_gc_guard!` before dereferencing `sptr` would be of no use.
///
/// Using the `rb_gc_guard!` macro has the following advantages:
///
/// - the intent of the macro use is clear.
///
/// - `rb_gc_guard!` only affects its call site, without negatively affecting other systems.
///
/// # Example
/// ```no_run
/// use rb_sys::{rb_utf8_str_new_cstr, rb_gc_guard};
///
/// let my_string = unsafe { rb_utf8_str_new_cstr("hello world\0".as_ptr() as _) };
/// let _ = rb_gc_guard!(my_string);
/// ```
#[macro_export]
macro_rules! rb_gc_guard {
    ($v:expr) => {{
        // This matches Ruby's RB_GC_GUARD implementation:
        //
        //   volatile VALUE *rb_gc_guarded_ptr = &(v);
        //   __asm__("" : : "m"(rb_gc_guarded_ptr));
        //
        // The empty asm with "m" (memory) constraint tells the compiler:
        // 1. The value must be in memory (not just a register)
        // 2. The compiler cannot reorder or eliminate this memory access
        //
        // In Rust, we achieve this by:
        // 1. Taking a reference to force stack allocation
        // 2. Using read_volatile to prevent optimization
        let rb_gc_guarded_ptr: *const $crate::VALUE = &$v;
        // SAFETY: rb_gc_guarded_ptr points to a valid, aligned VALUE on the
        // stack (created on the line above). The read is volatile to ensure
        // the compiler keeps the VALUE visible for conservative GC scanning.
        unsafe { std::ptr::read_volatile(rb_gc_guarded_ptr) }
    }};
}
