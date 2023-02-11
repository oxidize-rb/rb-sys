use std::collections::HashSet;

/// The Ruby headers to include in the bindings.
#[derive(Debug, Clone)]
pub struct RubyHeaders {
    headers: HashSet<&'static str>,
}

impl RubyHeaders {
    pub fn include(mut self, header: &'static str) -> Self {
        self.headers.insert(header);
        self
    }

    pub fn exclude(mut self, header: &'static str) -> Self {
        self.headers.remove(header);
        self
    }
}

impl Default for RubyHeaders {
    fn default() -> Self {
        static RUBY_HEADERS: [&str; 24] = [
            "ruby/debug.h",
            "ruby/defines.h",
            "ruby/encoding.h",
            "ruby/fiber/scheduler.h",
            "ruby/intern.h",
            "ruby/io.h",
            "ruby/memory_view.h",
            "ruby/missing.h",
            "ruby/onigmo.h",
            "ruby/oniguruma.h",
            "ruby/ractor.h",
            "ruby/random.h",
            "ruby/re.h",
            "ruby/regex.h",
            "ruby/ruby.h",
            "ruby/st.h",
            "ruby/thread.h",
            "ruby/thread_native.h",
            "ruby/util.h",
            "ruby/version.h",
            "ruby/vm.h",
            "ruby/win32.h",
            "ruby/io/buffer.h",
            "ruby/atomic.h",
        ];

        let headers = HashSet::from(RUBY_HEADERS);

        Self { headers }
    }
}

impl ToString for RubyHeaders {
    fn to_string(&self) -> String {
        let mut dot_h: String = "#include \"ruby.h\"\n".into();

        for header in &self.headers {
            let have_macro = format!("HAVE_{}", header.to_uppercase().replace('.', "_"));
            let have_macro = have_macro.replace('/', "_");

            dot_h.push_str(&format!(
                "#ifdef {}\n#include \"{}\"\n#endif\n",
                have_macro, header
            ));
        }

        dot_h
    }
}
