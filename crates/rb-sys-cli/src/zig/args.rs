//! Compiler and linker argument filtering for Zig compatibility.
//!
//! This module handles the transformation of arguments passed by cargo/cc-rs
//! to be compatible with Zig's compiler driver. Different platforms require
//! different filtering rules.

use super::target::{Arch, Env, Os, RustTarget};

/// Filters and transforms compiler/linker arguments for Zig compatibility.
pub struct ArgFilter<'a> {
    target: &'a RustTarget,
}

impl<'a> ArgFilter<'a> {
    /// Create a new argument filter for the given target.
    pub fn new(target: &'a RustTarget) -> Self {
        Self { target }
    }

    /// Filter compiler (CC/CXX) arguments.
    ///
    /// Handles architecture-specific `-march` rewriting and other
    /// compiler flag transformations.
    pub fn filter_cc_args(&self, args: &[String]) -> Vec<String> {
        let mut result = Vec::with_capacity(args.len());
        let mut iter = args.iter().peekable();

        while let Some(arg) = iter.next() {
            if let Some(filtered) = self.filter_cc_arg(arg, &mut iter) {
                result.extend(filtered);
            }
        }

        result
    }

    /// Filter linker arguments.
    ///
    /// Handles library substitutions, removal of unsupported flags,
    /// and platform-specific linker argument transformations.
    pub fn filter_link_args(&self, args: &[String]) -> Vec<String> {
        let mut result = Vec::with_capacity(args.len());
        let mut iter = args.iter().peekable();

        while let Some(arg) = iter.next() {
            if let Some(filtered) = self.filter_link_arg(arg, &mut iter) {
                result.extend(filtered);
            }
        }

        result
    }

    /// Filter a single CC argument.
    fn filter_cc_arg<'b, I>(
        &self,
        arg: &str,
        _iter: &mut std::iter::Peekable<I>,
    ) -> Option<Vec<String>>
    where
        I: Iterator<Item = &'b String>,
    {
        // ARM: remove -march (we use -mcpu instead)
        if self.target.arch == Arch::Arm && arg.starts_with("-march=") {
            return None;
        }

        // aarch64 Linux: rewrite -march flags
        if self.target.arch == Arch::Aarch64 && self.target.os == Os::Linux {
            if arg == "-march=armv8-a" {
                return Some(vec![
                    "-march=generic".to_string(),
                    "-Xassembler".to_string(),
                    "-march=armv8-a".to_string(),
                ]);
            }
            if let Some(suffix) = arg.strip_prefix("-march=armv8-a+") {
                // Replace simd with neon (suffix doesn't include the leading +)
                let suffix = suffix.replace("simd", "neon");
                return Some(vec![
                    "-march=generic".to_string(),
                    "-Xassembler".to_string(),
                    format!("-march=armv8-a+{}", suffix),
                ]);
            }
        }

        // aarch64 macOS: rewrite -march=armv8-a to apple_m1
        if self.target.arch == Arch::Aarch64 && self.target.os == Os::Darwin {
            if arg == "-march=armv8-a" {
                return Some(vec!["-march=apple_m1".to_string()]);
            }
        }

        // Pass through unchanged
        Some(vec![arg.to_string()])
    }

    /// Filter a single linker argument.
    fn filter_link_arg<'b, I>(
        &self,
        arg: &str,
        iter: &mut std::iter::Peekable<I>,
    ) -> Option<Vec<String>>
    where
        I: Iterator<Item = &'b String>,
    {
        // === Handle -Wl, prefixed args ===
        // When using ld.lld directly, we need to strip -Wl, and process the inner flags
        if let Some(inner) = arg.strip_prefix("-Wl,") {
            return self.filter_wl_arg(inner, iter);
        }

        // === Global filters (all platforms) ===

        // -lgcc_s: Zig provides compiler-rt, just remove it
        if arg == "-lgcc_s" {
            return None;
        }

        // Remove these unconditionally
        if arg.starts_with("--target=") {
            return None;
        }

        // Remove macOS-specific linker flags when NOT targeting macOS
        if self.target.os != Os::Darwin {
            // -dynamiclib is macOS-only (Linux/Windows use -shared)
            if arg == "-dynamiclib" {
                return Some(vec!["-shared".to_string()]);
            }
        }

        // === Windows GNU (MinGW) specific ===
        if self.target.os == Os::Windows {
            // -lgcc_eh → -lc++
            if arg == "-lgcc_eh" {
                return Some(vec!["-lc++".to_string()]);
            }

            // Remove MinGW-specific flags that Zig doesn't support
            if matches!(
                arg,
                "-lwindows"
                    | "-l:libpthread.a"
                    | "-lgcc"
                    | "-Wl,--disable-auto-image-base"
                    | "-Wl,--dynamicbase"
                    | "-Wl,--large-address-aware"
                    | "-lmsvcrt"
                    | "-Wl,--allow-shlib-undefined"
            ) {
                return None;
            }

            // Remove .def files
            if arg.ends_with(".def") {
                return None;
            }

            // Remove compiler_builtins.rlib (zig has compiler-rt)
            if arg.contains("compiler_builtins") && arg.ends_with(".rlib") {
                return None;
            }
        }

        // === Linux specific ===
        if self.target.os == Os::Linux {
            // When using ld.lld directly (not zig cc as linker driver), we need to
            // filter out system library flags. These libraries are provided by the
            // system at runtime - for Ruby extensions, Ruby already has them linked.
            // Zig's ld.lld doesn't have access to Zig's glibc shims, so we can't
            // resolve these at link time.
            if matches!(
                arg,
                "-lc" | "-lm" | "-ldl" | "-lpthread" | "-lrt" | "-lutil" | "-lgcc" | "-lgcc_s"
            ) {
                return None;
            }
        }

        // === musl specific ===
        if self.target.env == Env::Musl {
            // Remove self-contained CRT objects
            if (arg.contains("crt") || arg.contains("crti") || arg.contains("crtn"))
                && arg.ends_with(".o")
            {
                return None;
            }

            if arg == "-Wl,-melf_i386" {
                return None;
            }
        }

        // === macOS specific ===
        if self.target.os == Os::Darwin {
            // Remove -Wl,-exported_symbols_list and its argument
            if arg == "-Wl,-exported_symbols_list" {
                iter.next(); // skip the next argument (the symbols file path)
                return None;
            }

            if arg == "-Wl,-dylib" {
                return None;
            }

            // Keep -Wl,-framework (important for macOS)
        }

        // === ARM specific ===
        if self.target.arch == Arch::Arm {
            // Remove compiler_builtins.rlib (zig has compiler-rt)
            if arg.contains("compiler_builtins") && arg.ends_with(".rlib") {
                return None;
            }
        }

        // Pass through unchanged
        Some(vec![arg.to_string()])
    }

    /// Filter -Wl, prefixed arguments.
    ///
    /// When using ld.lld directly (not through a compiler driver), we need to:
    /// 1. Strip the -Wl, prefix
    /// 2. Split comma-separated args
    /// 3. Filter out unsupported flags
    /// 4. Return the remaining flags without -Wl, prefix
    fn filter_wl_arg<'b, I>(
        &self,
        inner: &str,
        iter: &mut std::iter::Peekable<I>,
    ) -> Option<Vec<String>>
    where
        I: Iterator<Item = &'b String>,
    {
        // Split by comma to handle -Wl,flag1,flag2,... format
        let parts: Vec<&str> = inner.split(',').collect();

        // Check for flags to remove entirely
        let first = parts.first().copied().unwrap_or("");

        // === Global removals ===
        if first == "--no-undefined-version"
            || first == "-znostart-stop-gc"
            || first == "--eh-frame-hdr"
            || (first == "-z" && parts.get(1) == Some(&"nostart-stop-gc"))
        {
            return None;
        }

        // === Windows MinGW flags to remove ===
        if self.target.os == Os::Windows {
            if first == "--dynamicbase"
                || first == "--disable-auto-image-base"
                || first == "--large-address-aware"
                || first == "--allow-shlib-undefined"
            {
                return None;
            }
        }

        // === musl-specific flags to remove ===
        if self.target.env == Env::Musl {
            if first == "-melf_i386" {
                return None;
            }
        }

        // === macOS-specific flags handling ===
        if self.target.os == Os::Darwin {
            // For Darwin, keep these flags but strip -Wl, prefix
            // -dylib, -exported_symbols_list, etc. are valid for ld64.lld
            if first == "-exported_symbols_list" {
                // This flag takes a path argument as the next separate arg, consume it
                iter.next();
                return None;
            }
            if first == "-dylib" {
                // ld64.lld handles -dylib differently, remove it
                return None;
            }
        } else {
            // For non-Darwin, remove Darwin-specific flags
            if first == "-multiply_defined"
                || first == "-undefined"
                || first == "-dylib"
                || first == "-exported_symbols_list"
                || first == "-install_name"
                || first == "-compatibility_version"
                || first == "-current_version"
            {
                return None;
            }
        }

        // Pass through the inner args (without -Wl, prefix) since we're calling ld directly
        Some(parts.iter().map(|s| s.to_string()).collect())
    }
}

/// Filter AR (archiver) arguments for Zig compatibility.
///
/// Zig's ar (llvm-ar) doesn't support all GNU ar modifiers.
/// This function removes unsupported modifiers from operation strings.
pub fn filter_ar_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            // Check if this looks like an operation+modifiers string (e.g., "cqn", "rcsD")
            // AR operations: c (create), d (delete), m (move), p (print), q (quick append),
            //               r (replace), s (symbol index), t (table), x (extract)
            if arg.len() > 1
                && arg
                    .chars()
                    .next()
                    .map(|c| "cdmpqrstx".contains(c))
                    .unwrap_or(false)
                && arg.chars().skip(1).all(|c| c.is_ascii_alphabetic())
            {
                // Remove 'n' modifier (don't add symbol table) - not supported by llvm-ar
                arg.replace('n', "")
            } else {
                arg.clone()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Global filter tests ===

    #[test]
    fn test_global_lgcc_s_removed() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&["-lgcc_s".to_string(), "-lfoo".to_string()]);
        // -lgcc_s is removed (zig provides compiler-rt)
        // Note: -lc would also be removed for Linux, so we use -lfoo
        assert_eq!(result, vec!["-lfoo"]);
    }

    #[test]
    fn test_global_remove_target_flag() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "--target=x86_64-unknown-linux-gnu".to_string(),
            "-lfoo".to_string(),
        ]);
        // Note: -lc would be removed for Linux, so we use -lfoo
        assert_eq!(result, vec!["-lfoo"]);
    }

    #[test]
    fn test_global_remove_no_undefined_version() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-Wl,--no-undefined-version".to_string(),
            "-lfoo".to_string(),
        ]);
        // -Wl,--no-undefined-version is removed entirely
        // Note: -lm would be removed for Linux, so we use -lfoo
        assert_eq!(result, vec!["-lfoo"]);
    }

    #[test]
    fn test_global_remove_eh_frame_hdr() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result =
            filter.filter_link_args(&["-Wl,--eh-frame-hdr".to_string(), "-lfoo".to_string()]);
        // -Wl,--eh-frame-hdr is removed entirely
        // Note: -lc would be removed for Linux, so we use -lfoo
        assert_eq!(result, vec!["-lfoo"]);
    }

    #[test]
    fn test_linux_remove_system_libs() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-lc".to_string(),
            "-lm".to_string(),
            "-ldl".to_string(),
            "-lpthread".to_string(),
            "-lrt".to_string(),
            "-lutil".to_string(),
            "-lfoo".to_string(),
        ]);
        // System libs are removed for Linux (resolved at runtime)
        assert_eq!(result, vec!["-lfoo"]);
    }

    // === Windows GNU (MinGW) tests ===

    #[test]
    fn test_windows_lgcc_eh_to_lcpp() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&["-lgcc_eh".to_string()]);
        assert_eq!(result, vec!["-lc++"]);
    }

    #[test]
    fn test_windows_remove_mingw_flags() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-lwindows".to_string(),
            "-lkernel32".to_string(),
            "-Wl,--dynamicbase".to_string(),
        ]);
        assert_eq!(result, vec!["-lkernel32"]);
    }

    #[test]
    fn test_windows_remove_def_files() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result =
            filter.filter_link_args(&["/path/to/exports.def".to_string(), "-luser32".to_string()]);
        assert_eq!(result, vec!["-luser32"]);
    }

    #[test]
    fn test_windows_remove_compiler_builtins() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "/path/to/libcompiler_builtins-abc123.rlib".to_string(),
            "-lkernel32".to_string(),
        ]);
        assert_eq!(result, vec!["-lkernel32"]);
    }

    #[test]
    fn test_windows_aarch64_gnullvm() {
        let target = RustTarget::parse("aarch64-pc-windows-gnullvm").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-lgcc_eh".to_string(),
            "-lwindows".to_string(),
            "-lkernel32".to_string(),
        ]);
        assert_eq!(result, vec!["-lc++", "-lkernel32"]);
    }

    // === musl tests ===

    #[test]
    fn test_musl_remove_crt_objects() {
        let target = RustTarget::parse("x86_64-unknown-linux-musl").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "/path/to/crt1.o".to_string(),
            "-lfoo".to_string(),
            "/path/to/crtn.o".to_string(),
        ]);
        // CRT objects are removed, -lc would also be removed (system lib)
        assert_eq!(result, vec!["-lfoo"]);
    }

    #[test]
    fn test_musl_remove_crti() {
        let target = RustTarget::parse("aarch64-unknown-linux-musl").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&["/path/to/crti.o".to_string(), "-lfoo".to_string()]);
        // CRT objects are removed, -lpthread would also be removed (system lib)
        assert_eq!(result, vec!["-lfoo"]);
    }

    #[test]
    fn test_musl_remove_melf_i386() {
        let target = RustTarget::parse("x86_64-unknown-linux-musl").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&["-Wl,-melf_i386".to_string(), "-lfoo".to_string()]);
        // -Wl,-melf_i386 is removed, -lc would also be removed (system lib)
        assert_eq!(result, vec!["-lfoo"]);
    }

    // === macOS tests ===

    #[test]
    fn test_macos_remove_exported_symbols_list() {
        let target = RustTarget::parse("x86_64-apple-darwin").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-Wl,-exported_symbols_list".to_string(),
            "/path/to/symbols.txt".to_string(),
            "-lSystem".to_string(),
        ]);
        assert_eq!(result, vec!["-lSystem"]);
    }

    #[test]
    fn test_macos_remove_dylib() {
        let target = RustTarget::parse("aarch64-apple-darwin").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&["-Wl,-dylib".to_string(), "-lSystem".to_string()]);
        assert_eq!(result, vec!["-lSystem"]);
    }

    #[test]
    fn test_macos_keep_framework() {
        let target = RustTarget::parse("x86_64-apple-darwin").unwrap();
        let filter = ArgFilter::new(&target);
        let result =
            filter.filter_link_args(&["-Wl,-framework".to_string(), "CoreFoundation".to_string()]);
        // -Wl, prefix is stripped since we call ld64.lld directly
        assert_eq!(result, vec!["-framework", "CoreFoundation"]);
    }

    // === ARM CC tests ===

    #[test]
    fn test_arm_remove_march() {
        let target = RustTarget::parse("arm-unknown-linux-gnueabihf").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_cc_args(&["-march=armv6".to_string(), "-O2".to_string()]);
        assert_eq!(result, vec!["-O2"]);
    }

    #[test]
    fn test_arm_remove_compiler_builtins() {
        let target = RustTarget::parse("arm-unknown-linux-gnueabihf").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "/path/to/libcompiler_builtins-xyz.rlib".to_string(),
            "-lfoo".to_string(),
        ]);
        // compiler_builtins is removed, -lc would also be removed (system lib)
        assert_eq!(result, vec!["-lfoo"]);
    }

    // === aarch64 Linux CC tests ===

    #[test]
    fn test_aarch64_linux_march_rewrite() {
        let target = RustTarget::parse("aarch64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_cc_args(&["-march=armv8-a".to_string()]);
        assert_eq!(
            result,
            vec!["-march=generic", "-Xassembler", "-march=armv8-a"]
        );
    }

    #[test]
    fn test_aarch64_linux_march_with_crypto() {
        let target = RustTarget::parse("aarch64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_cc_args(&["-march=armv8-a+crypto".to_string()]);
        assert_eq!(
            result,
            vec!["-march=generic", "-Xassembler", "-march=armv8-a+crypto"]
        );
    }

    #[test]
    fn test_aarch64_linux_march_simd_to_neon() {
        let target = RustTarget::parse("aarch64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_cc_args(&["-march=armv8-a+simd".to_string()]);
        assert_eq!(
            result,
            vec!["-march=generic", "-Xassembler", "-march=armv8-a+neon"]
        );
    }

    // === aarch64 macOS CC tests ===

    #[test]
    fn test_aarch64_macos_march_rewrite() {
        let target = RustTarget::parse("aarch64-apple-darwin").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_cc_args(&["-march=armv8-a".to_string()]);
        assert_eq!(result, vec!["-march=apple_m1"]);
    }

    #[test]
    fn test_x86_64_macos_no_march_rewrite() {
        let target = RustTarget::parse("x86_64-apple-darwin").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_cc_args(&["-march=x86-64".to_string()]);
        assert_eq!(result, vec!["-march=x86-64"]);
    }

    // === AR filter tests ===

    #[test]
    fn test_ar_remove_n_modifier() {
        let result = filter_ar_args(&["cqn".to_string(), "libfoo.a".to_string()]);
        assert_eq!(result, vec!["cq", "libfoo.a"]);
    }

    #[test]
    fn test_ar_remove_n_from_rcsn() {
        let result = filter_ar_args(&["rcsn".to_string(), "libbar.a".to_string()]);
        assert_eq!(result, vec!["rcs", "libbar.a"]);
    }

    #[test]
    fn test_ar_keep_other_modifiers() {
        let result = filter_ar_args(&["rcsD".to_string(), "libfoo.a".to_string()]);
        assert_eq!(result, vec!["rcsD", "libfoo.a"]);
    }

    #[test]
    fn test_ar_dont_modify_paths() {
        let result = filter_ar_args(&["rcs".to_string(), "/path/with/n/libfoo.a".to_string()]);
        assert_eq!(result, vec!["rcs", "/path/with/n/libfoo.a"]);
    }

    // === Passthrough tests ===

    #[test]
    fn test_passthrough_normal_args() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-L/usr/lib".to_string(),
            "-lssl".to_string(),
            "-lcrypto".to_string(),
        ]);
        assert_eq!(result, vec!["-L/usr/lib", "-lssl", "-lcrypto"]);
    }

    #[test]
    fn test_passthrough_cc_args() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_cc_args(&[
            "-I/usr/include".to_string(),
            "-O2".to_string(),
            "-Wall".to_string(),
        ]);
        assert_eq!(result, vec!["-I/usr/include", "-O2", "-Wall"]);
    }

    // === macOS linker flag filtering for non-Darwin targets ===

    #[test]
    fn test_linux_removes_macos_multiply_defined() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-Wl,-multiply_defined,suppress".to_string(),
            "-lfoo".to_string(),
        ]);
        // -Wl,-multiply_defined,suppress is removed entirely for non-Darwin
        // Note: -lc would also be removed (system lib)
        assert_eq!(result, vec!["-lfoo"]);
    }

    #[test]
    fn test_linux_removes_macos_undefined_dynamic_lookup() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-Wl,-undefined,dynamic_lookup".to_string(),
            "-lfoo".to_string(),
        ]);
        // -Wl,-undefined,dynamic_lookup is removed entirely for non-Darwin
        // Note: -lm would also be removed (system lib)
        assert_eq!(result, vec!["-lfoo"]);
    }

    #[test]
    fn test_linux_converts_dynamiclib_to_shared() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-dynamiclib".to_string(),
            "-o".to_string(),
            "libfoo.so".to_string(),
        ]);
        assert_eq!(result, vec!["-shared", "-o", "libfoo.so"]);
    }

    #[test]
    fn test_darwin_keeps_macos_linker_flags() {
        let target = RustTarget::parse("aarch64-apple-darwin").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-Wl,-multiply_defined,suppress".to_string(),
            "-Wl,-undefined,dynamic_lookup".to_string(),
            "-lSystem".to_string(),
        ]);
        // Darwin should keep these flags (stripped of -Wl, prefix for direct ld use)
        assert!(result.contains(&"-multiply_defined".to_string()));
        assert!(result.contains(&"suppress".to_string()));
        assert!(result.contains(&"-undefined".to_string()));
        assert!(result.contains(&"dynamic_lookup".to_string()));
        assert!(result.contains(&"-lSystem".to_string()));
    }

    #[test]
    fn test_windows_removes_macos_linker_flags() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        let filter = ArgFilter::new(&target);
        let result = filter.filter_link_args(&[
            "-Wl,-multiply_defined,suppress".to_string(),
            "-lkernel32".to_string(),
        ]);
        assert_eq!(result, vec!["-lkernel32"]);
    }
}
