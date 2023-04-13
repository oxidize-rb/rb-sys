use rb_sys_build::{utils::is_mswin_or_mingw, RbConfig};

pub fn is_global_allocator_enabled() -> bool {
    is_env_variable_defined("CARGO_FEATURE_GLOBAL_ALLOCATOR")
}

pub fn is_ruby_macros_enabled() -> bool {
    if is_mswin_or_mingw() {
        return false;
    }

    !is_linting() && is_env_variable_defined("CARGO_FEATURE_RUBY_MACROS")
}

pub fn is_gem_enabled() -> bool {
    cfg!(rb_sys_gem)
}

pub fn is_no_link_ruby_enabled() -> bool {
    is_env_variable_defined("CARGO_FEATURE_NO_LINK_RUBY")
}

pub fn is_debug_build_enabled() -> bool {
    if is_linting() {
        return false;
    }

    println!("cargo:rerun-if-env-changed=RB_SYS_DEBUG_BUILD");

    is_env_variable_defined("RB_SYS_DEBUG_BUILD")
}

pub fn is_ruby_static_enabled(rbconfig: &RbConfig) -> bool {
    println!("cargo:rerun-if-env-changed=RUBY_STATIC");

    match std::env::var("RUBY_STATIC") {
        Ok(val) => val == "true" || val == "1",
        _ => {
            is_env_variable_defined("CARGO_FEATURE_RUBY_STATIC")
                || rbconfig.get("ENABLE_SHARED") == "no"
        }
    }
}

pub fn is_link_ruby_enabled() -> bool {
    if is_linting() {
        return false;
    }

    if is_no_link_ruby_enabled() {
        false
    } else if is_mswin_or_mingw() {
        true
    } else if is_gem_enabled() {
        if is_env_variable_defined("CARGO_FEATURE_LINK_RUBY") {
            let msg = "
                The `gem` and `link-ruby` features are mutually exclusive on this
                platform, since the libruby symbols will be available at runtime.

                If you for some reason want to dangerously link libruby for your gem
                (*not recommended*), you can remove the `gem` feature and add this
                to your `Cargo.toml`:

                [dependencies.rb-sys]
                features = [\"link-ruby\"] # Living dangerously!
            "
            .split('\n')
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n");

            eprintln!("ERROR: {}", msg);
            std::process::exit(1);
        } else {
            false
        }
    } else {
        is_env_variable_defined("CARGO_FEATURE_LINK_RUBY")
    }
}

fn is_env_variable_defined(name: &str) -> bool {
    std::env::var(name).is_ok()
}

fn is_linting() -> bool {
    let clippy = match std::env::var_os("CARGO_CFG_FEATURE") {
        Some(val) => val.to_str().unwrap_or("").contains("clippy"),
        _ => false,
    };

    let rust_analyzer = match std::env::var_os("RUSTC_WRAPPER") {
        Some(val) => val.to_str().unwrap_or("").contains("rust-analyzer"),
        _ => false,
    };

    clippy || rust_analyzer
}
