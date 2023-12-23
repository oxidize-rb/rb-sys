use robusta_jni::convert::{Signature, TryFromJavaValue, TryIntoJavaValue};
use robusta_jni::jni::{
    objects::{JClass, JString},
    strings::JNIString,
    sys::{jint, JNI_ERR, JNI_VERSION_1_4},
    JNIEnv, JavaVM, NativeMethod,
};
use std::os::raw::c_void;

extern "system" fn pub_reverse<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    input: <String as TryFromJavaValue<'local, 'local>>::Source,
) -> <String as TryIntoJavaValue<'local>>::Target {
    let java_string_res: robusta_jni::jni::errors::Result<String> = TryFromJavaValue::try_from(input, &env);
    match java_string_res {
        Ok(java_string) => {
            let reversed = java_string.chars().rev().collect::<String>();
            let reversed_res: robusta_jni::jni::errors::Result<<String as TryIntoJavaValue>::Target> =
                TryIntoJavaValue::try_into(reversed, &env);
            match reversed_res {
                Ok(conv_res) => {
                    return conv_res;
                }
                Err(err) => {
                    // No need to handle err, ClassNotFoundException will be thrown implicitly
                    let _ = env.throw_new("java/lang/RuntimeException", format!("{:?}", err));
                }
            }
        }
        Err(err) => {
            // No need to handle err, ClassNotFoundException will be thrown implicitly
            let _ = env.throw_new("java/lang/RuntimeException", format!("{:?}", err));
        }
    }
    JString::from(std::ptr::null_mut())
}

/// This function is executed on loading native library by JVM.
/// It initializes the cache of method and class references.
#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn JNI_OnLoad<'local>(vm: JavaVM, _: *mut c_void) -> jint {
    let Ok(env) = vm.get_env() else {
        return JNI_ERR;
    };
    let Ok(clazz) = env.find_class("rbsys/rust_reverse/RustReverse") else {
        return JNI_ERR;
    };
    let reverse_func = pub_reverse
        as unsafe extern "system" fn(
        env: JNIEnv<'local>,
        _class: JClass<'local>,
        input: JString<'local>,
    ) -> JString<'local>;
    let reverse_ptr = reverse_func as *mut c_void;
    let reverse_method = NativeMethod {
        name: JNIString::from("reverseNative"),
        sig: JNIString::from(format!(
            "({}){}",
            <JString as Signature>::SIG_TYPE,
            <JString as Signature>::SIG_TYPE
        )),
        fn_ptr: reverse_ptr,
    };
    let Ok(_) = env.register_native_methods(clazz, &[reverse_method]) else {
        return JNI_ERR;
    };
    JNI_VERSION_1_4
}
