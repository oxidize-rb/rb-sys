use rb_sys::*;

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_rarray_len_small() {
    let rstr: VALUE = rstring!("foo");
    let rarray = unsafe { rb_ary_new() };
    unsafe { rb_ary_push(rarray, rstr) };

    assert_eq!(unsafe { RARRAY_LEN(rarray) }, 1);
}

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_rarray_len_large() {
    let rarray = unsafe { rb_ary_new() };

    for _ in (0..100).into_iter() {
        let rstr: VALUE = rstring!("foo");
        unsafe { rb_ary_push(rarray, rstr) };
    }

    assert_eq!(unsafe { RARRAY_LEN(rarray) }, 100);
}
