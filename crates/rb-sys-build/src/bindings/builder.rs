use crate::{bindings::cfg::extract, rb_config::Library};

use super::{
    cfg::Item,
    docs::{DeprecationWarnings, DocCallbacks},
    filter::RemoveDefinesFilter,
    link_directives::AddRubyLinkDirectives,
    native_types::RemapNativeTypes,
    ruby_headers::RubyHeaders,
};
use bindgen::{callbacks::ParseCallbacks, CargoCallbacks};
use quote::ToTokens;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Debug,
    fs::File,
    io::Write,
    path::PathBuf,
    process::Stdio,
};
use syn::visit_mut::VisitMut;

/// A builder for generating bindings.
pub struct Builder {
    bindgen: bindgen::Builder,
    ruby_headers: RubyHeaders,
    blocklist_groups: HashSet<BindgenGroups>,
    ast_transforms: HashMap<&'static str, Box<dyn VisitMut>>,
    rustfmt: bool,
    parse_callbacks: HashMap<&'static str, Box<dyn ParseCallbacks>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Create a new builder.
    pub fn new() -> Self {
        let mut ast_transforms: HashMap<&'static str, Box<dyn VisitMut>> = HashMap::new();

        ast_transforms.insert("deprecation_warnings", Box::new(DeprecationWarnings));
        ast_transforms.insert("remove_defines", Box::new(RemoveDefinesFilter));
        ast_transforms.insert("remap_native_types", Box::new(RemapNativeTypes));

        Self {
            parse_callbacks: Default::default(),
            bindgen: default_bindgen(),
            rustfmt: true,
            blocklist_groups: HashSet::from([
                BindgenGroups::RbImpls,
                BindgenGroups::DeprecatedTypes,
            ]),
            ruby_headers: RubyHeaders::default(),
            ast_transforms,
        }
    }

    /// Enable layout tests in the bindings.
    pub fn layout_tests(mut self, doit: bool) -> Self {
        self.bindgen = self.bindgen.layout_tests(doit);
        self
    }

    /// Generate documentation for the bindings.
    pub fn docs(mut self, doit: bool) -> Self {
        if doit {
            self.parse_callbacks.insert("docs", Box::new(DocCallbacks));
        } else {
            self.parse_callbacks.remove("docs");
        }
        self.bindgen = self.bindgen.generate_comments(doit);

        self
    }

    /// Generate bindings for deprecated types.
    pub fn deprecated_types(mut self, doit: bool) -> Self {
        if doit {
            self.blocklist_groups
                .remove(&BindgenGroups::DeprecatedTypes);
        } else {
            self.blocklist_groups.insert(BindgenGroups::DeprecatedTypes);
        }
        self
    }

    /// Generate bindings for the `rbimpls` module.
    pub fn rbimpls(mut self, doit: bool) -> Self {
        if doit {
            self.blocklist_groups.remove(&BindgenGroups::RbImpls);
        } else {
            self.blocklist_groups.insert(BindgenGroups::RbImpls);
        }

        self
    }

    /// Generate `impl Debug` for the bindings.
    pub fn impl_debug(mut self, doit: bool) -> Self {
        self.bindgen = self.bindgen.impl_debug(doit);
        self
    }

    /// Link statically to the Ruby library.
    pub fn add_link_ruby_directive(mut self, lib: Library) -> Self {
        let name = lib.name().to_string();
        let kind = lib
            .kind()
            .expect("kind is required for linking")
            .to_string();

        self.ast_transforms.insert(
            "linkage",
            Box::new(AddRubyLinkDirectives::new(&name, &kind)),
        );

        self
    }

    /// Add a Ruby header to include when generating bindings.
    pub fn include_ruby_header(mut self, header: &'static str) -> Self {
        self.ruby_headers = self.ruby_headers.include(header);
        self
    }

    /// Do not include this Ruby header when generating the bindings.
    pub fn exclude_ruby_header(mut self, header: &'static str) -> Self {
        self.ruby_headers = self.ruby_headers.exclude(header);
        self
    }

    /// Add an include path for the Ruby headers.
    pub fn include<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.bindgen = self
            .bindgen
            .clang_arg(format!("-I{}", path.into().to_str().unwrap()));
        self
    }

    /// Add a C flag for the Ruby headers.
    pub fn append_cflags<T: AsRef<str>>(mut self, flags: &[T]) -> Self {
        self.bindgen = self.bindgen.clang_args(flags);
        self
    }

    /// Add a C flag for the Ruby headers.
    pub fn append_cflag<T: Into<String>>(mut self, flag: T) -> Self {
        self.bindgen = self.bindgen.clang_arg(flag);
        self
    }

    /// Add a documentation comment to the bindings.
    pub fn doc_comment<T: Into<String>>(mut self, comment: T) -> Self {
        self.bindgen = self.bindgen.raw_line(comment);
        self
    }

    /// Run bindgen with the given configuration, and return a result containing
    /// to Rust code and parsed configuration as a hash map.
    pub fn generate(self) -> Result<Bindings, Box<dyn std::error::Error>> {
        let ruby_headers = self.ruby_headers.to_string();
        let mut bindgen = self.bindgen;

        for group in self.blocklist_groups {
            bindgen = group.apply_to_bindgen(bindgen);
        }

        let mut sorted_callbacks: Vec<_> = self
            .parse_callbacks
            .iter()
            .map(|(k, v)| (*k, v))
            .collect::<Vec<_>>();

        sorted_callbacks.sort_by_key(|(k1, _)| match *k1 {
            "docs" => 1,
            _ => 0,
        });

        for (_name, callback) in self.parse_callbacks {
            bindgen = bindgen.parse_callbacks(callback);
        }

        bindgen = bindgen.header_contents("wrapper.h", &ruby_headers);

        if std::env::var_os("RB_SYS_DEBUG_BUILD").is_some() {
            dbg!("Bindgen config:", &bindgen);
        }

        let bindings = bindgen.generate()?.to_string();
        let mut syntax = syn::parse_file(&bindings)?;
        let cfg = extract(&syntax)?;

        let mut sorted_transforms: Vec<_> = self
            .ast_transforms
            .iter()
            .map(|(k, v)| (*k, v))
            .collect::<Vec<_>>();

        sorted_transforms.sort_by_key(|(k1, _)| match *k1 {
            "remove_defines" => 0,
            "remap_native_types" => 0,
            "deprecation_warnings" => 1,
            "linkage" => 2,
            _ => 1,
        });

        for (_, mut transform) in self.ast_transforms {
            transform.visit_file_mut(&mut syntax);
        }

        let mut code = syntax.to_token_stream().to_string();

        if self.rustfmt {
            code = run_rustfmt(&code)?;
        }

        Ok(Bindings { code, cfg })
    }

    /// Print cargo directives for the Ruby library (i.e. `cargo:rerun-if-changed`).
    pub fn print_cargo_directives(mut self, doit: bool) -> Self {
        if doit {
            self.parse_callbacks
                .insert("cargo", Box::new(CargoCallbacks));
        } else {
            self.parse_callbacks.remove("cargo");
        }

        self
    }

    /// Transform the AST before it is written to output.
    pub fn register_ast_transform(
        mut self,
        name: &'static str,
        transform: impl VisitMut + 'static,
    ) -> Self {
        self.ast_transforms.insert(name, Box::new(transform));
        self
    }
}

impl Debug for Builder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builder")
            .field("bindgen", &self.bindgen)
            .field("ruby_headers", &self.ruby_headers)
            .field("blocklist_groups", &self.blocklist_groups)
            .field("ast_transforms", &"HashMap<...>")
            .field("rustfmt", &self.rustfmt)
            .field("parse_callbacks", &self.parse_callbacks)
            .finish()
    }
}

fn default_bindgen() -> bindgen::Builder {
    bindgen::Builder::default()
        .use_core()
        .clang_args(default_cflags())
        .rustfmt_bindings(false) // We use syn so this is pointless
        .rustified_enum(".*")
        .sort_semantically(true)
        .no_copy("rb_data_type_struct")
        .derive_eq(true)
        .derive_debug(true)
        .layout_tests(false)
        .size_t_is_usize(false)
        .merge_extern_blocks(true)
        .blocklist_type("VALUE") // Do this manually
        .blocklist_type("ID") // Do this manually
        .blocklist_item("^__darwin_pthread.*")
        .blocklist_item("^_opaque_pthread.*")
        .blocklist_item("^pthread_.*")
        .blocklist_item("^rb_native.*")
        .allowlist_file(".*ruby.*")
        .blocklist_item("ruby_abi_version")
        .blocklist_function("^__.*")
        .blocklist_item("RData")
        .blocklist_type("stat")
        .blocklist_function("rb_clear_constant_cache")
        .blocklist_function("rb_unexpected_type")
        .blocklist_function("rb_unexpected_type")
        .blocklist_function("rb_ruby_verbose_ptr")
        .blocklist_function("rb_ruby_debug_ptr")
        .blocklist_function("rb_io_extract_encoding_option")
        .blocklist_function("rb_check_type")
        .blocklist_function("rb_debug_rstring_null_ptr")
        .blocklist_function("rb_scan_args_bad_format")
        .blocklist_function("rb_scan_args_length_mismatch")
        .blocklist_type(
            "(time|mode|pid|off|suseconds|blkcnt|blksize|dev|uid|gid|nlink|ssize|size)_t",
        )
        .blocklist_type("^__darwin_.*_t$")
        .blocklist_type("__u?int\\d\\d_t")
        .generate_comments(false)
        .impl_debug(false)
}

/// The result of a binding generation, containing the generated code and parsed
/// configuration values.
#[derive(Debug)]
pub struct Bindings {
    code: String,
    cfg: Vec<Item>,
}

impl Bindings {
    /// Get the generated Rust code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Get the cargo configuration directives.
    pub fn cfg(&self) -> &[Item] {
        &self.cfg
    }

    /// Write the generated Rust code to a file.
    pub fn write_code_to_file(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        file.write_all(self.code.as_bytes())?;
        Ok(())
    }

    /// Write the generated Rust code to a file.
    pub fn write_code_to<T>(&self, io: &mut T) -> Result<(), Box<dyn Error>>
    where
        T: Write,
    {
        io.write_all(self.code.as_bytes())?;
        Ok(())
    }

    /// Write the rustc-cfg directives to a file (e.g. `cargo:rustc-cfg=use_transient_heap`)
    pub fn write_rustc_cfg_to<T>(&self, io: &mut T) -> Result<(), Box<dyn Error>>
    where
        T: Write,
    {
        for item in self.cfg.iter() {
            writeln!(io, "{}", item.as_cargo_cfg())?;
        }
        Ok(())
    }

    //// Write the parsed cargo configuration to a file (e.g. `cargo:defines_use_flonum=true`)
    pub fn write_cargo_cfg_to<T>(&self, io: &mut T) -> Result<(), Box<dyn Error>>
    where
        T: Write,
    {
        for item in self.cfg.iter() {
            writeln!(io, "{}", item.as_cargo_cfg())?;
        }
        Ok(())
    }
}

/// Logical groups of things to block or allow in the bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindgenGroups {
    RbImpls,
    DeprecatedTypes,
}

impl BindgenGroups {
    /// Apply this group to a bindgen builder.
    fn apply_to_bindgen(self, bindgen: bindgen::Builder) -> bindgen::Builder {
        match self {
            BindgenGroups::DeprecatedTypes => bindgen
                .blocklist_type("^ruby_fl_type.*")
                .blocklist_type("^_bindgen_ty_9.*"),
            BindgenGroups::RbImpls => bindgen
                .blocklist_function("^rbimpl_.*")
                .blocklist_function("^RBIMPL_.*"),
        }
    }
}

pub fn default_cflags() -> &'static [&'static str] {
    if cfg!(target_os = "openbsd") {
        &["-fdeclspec"]
    } else {
        &["-fms-extensions"]
    }
}

fn run_rustfmt(code: &str) -> Result<String, Box<dyn Error>> {
    let mut cmd = std::process::Command::new("rustfmt");
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(std::process::Stdio::inherit());
    cmd.arg("--edition=2018");
    cmd.arg("--emit=stdout");
    let mut child = cmd.spawn()?;
    let code = code.to_string();

    if let Some(mut stdin) = child.stdin.take() {
        std::thread::spawn(move || stdin.write_all(code.as_bytes()));
    }

    let output = child.wait_with_output()?;
    String::from_utf8(output.stdout).map_err(|_| "rustfmt output is not utf8".into())
}
