use crate::{rb_funcall_typed, rstring_to_string};
use rb_sys::{
    rb_ary_join, rb_class2name, rb_obj_class, rb_str_new,
    ruby_value_type::{RUBY_T_ARRAY, RUBY_T_STRING},
    RB_TYPE_P, VALUE,
};
use std::ffi::CStr;

/// A simple wrapper around a Ruby exception that provides some convenience
/// methods for testing.
#[derive(Clone, Eq, PartialEq)]
pub struct RubyException {
    value: VALUE,
}

impl RubyException {
    /// Creates a new Ruby exception from a Ruby value.
    pub fn new(value: VALUE) -> Self {
        Self { value }
    }

    /// Get the message of the Ruby exception.
    pub fn message(&self) -> Option<String> {
        unsafe {
            rb_funcall_typed!(self.value, "message", [], RUBY_T_STRING)
                .map(|mut message| rstring_to_string!(message))
        }
    }

    /// Get the full message of the Ruby exception.
    pub fn full_message(&self) -> Option<String> {
        unsafe {
            if let Some(mut message) =
                rb_funcall_typed!(self.value, "full_message", [], RUBY_T_STRING)
            {
                let message = rstring_to_string!(message);
                Some(message.trim_start_matches("-e: ").to_string())
            } else {
                None
            }
        }
    }

    /// Get the backtrace string of the Ruby exception.
    pub fn backtrace(&self) -> Option<String> {
        unsafe {
            if let Some(backtrace) = rb_funcall_typed!(self.value, "backtrace", [], RUBY_T_ARRAY) {
                let mut backtrace = rb_ary_join(backtrace, rb_str_new("\n".as_ptr() as _, 1));
                let backtrace = rstring_to_string!(backtrace);

                if backtrace.is_empty() {
                    return None;
                }

                Some(backtrace)
            } else {
                None
            }
        }
    }

    /// Get the inspect string of the Ruby exception.
    pub fn inspect(&self) -> String {
        unsafe {
            if let Some(mut inspect) = rb_funcall_typed!(self.value, "inspect", [], RUBY_T_STRING) {
                rstring_to_string!(inspect)
            } else {
                format!("<no inspect: {:?}>", self.value)
            }
        }
    }

    /// Get the class name of the Ruby exception.
    pub fn classname(&self) -> String {
        unsafe {
            let classname = rb_class2name(rb_obj_class(self.value));
            CStr::from_ptr(classname).to_string_lossy().into_owned()
        }
    }
}

// impl Drop for RubyException {
//     fn drop(&mut self) {
//         rb_sys::rb_gc_guard!(self.value);
//     }
// }

impl std::fmt::Debug for RubyException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.message();
        let klass = self.classname();
        let bt = self.backtrace();

        if let Some(full_message) = self.full_message() {
            return f.write_str(&full_message);
        }

        if let Some(message) = message {
            f.write_str(&message)?;
        } else {
            f.write_str("<no message>")?;
        }

        f.write_fmt(format_args!(" ({}):\n", klass))?;

        if let Some(bt) = bt {
            f.write_str(&bt)?;
        } else {
            f.write_str("<no backtrace>")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{protect, with_ruby_vm};
    use rb_sys::rb_eval_string;

    #[test]
    fn test_exception() -> Result<(), Box<dyn std::error::Error>> {
        with_ruby_vm(|| {
            let exception = protect(|| unsafe {
                rb_eval_string("raise 'oh no'\0".as_ptr() as _);
            })
            .unwrap_err();

            assert_eq!("RuntimeError", exception.classname());
            assert_eq!("oh no", exception.message().unwrap());
            #[cfg(ruby_gt_2_4)]
            {
                let message = exception.full_message().unwrap();
                assert!(message.contains("eval:1:in "), "message: {}", message);
            }
        })
    }
}
