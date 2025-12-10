mod assets;
mod blake3_hash;
mod build;
pub mod generated_mappings;
mod libclang;
mod platform;
mod sysroot;
mod toolchain;
mod tools;
mod zig;

use anyhow::Result;
use assets::AssetManager;
use clap::{Parser, Subcommand};
use tools as tool_helpers;
use tracing_subscriber::{fmt, EnvFilter};
use zig::ZigShim;

/// Setup logging based on verbose flag or RUST_LOG environment variable
fn setup_logging(verbose: bool) {
    // RUST_LOG env var takes precedence if set
    let filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else if verbose {
        // Verbose mode: show debug logs for rb-sys-cli
        EnvFilter::new("rb_sys_cli=debug,rb_sys_build=debug")
    } else {
        // Default: info level
        EnvFilter::new("rb_sys_cli=info,rb_sys_build=info")
    };

    fmt().with_env_filter(filter).with_target(false).init();
}

#[derive(Parser)]
#[command(name = "cargo-gem")]
#[command(bin_name = "cargo-gem")]
#[command(version, about = "Cross-compile Rust native gems with ease", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a native gem for a specific target platform
    #[command(alias = "b")]
    Build(build::BuildConfig),

    /// List supported target platforms
    #[command(alias = "ls")]
    List,

    /// Inspect embedded tooling
    Tools,

    /// Test macOS SDK embedding
    #[command(hide = true)]
    TestSdk,

    /// Cache management
    Cache {
        #[command(subcommand)]
        action: CacheCommands,
    },

    /// Internal: Zig CC wrapper (called by shims)
    #[command(hide = true)]
    ZigCc(zig::tools::ZigCcArgs),

    /// Internal: Zig C++ wrapper (called by shims)
    #[command(hide = true)]
    ZigCxx(zig::tools::ZigCcArgs),

    /// Internal: Zig AR wrapper (called by shims)
    #[command(hide = true)]
    ZigAr(zig::tools::ZigArArgs),

    /// Internal: Zig LD wrapper (called by shims)
    #[command(hide = true)]
    ZigLd(zig::tools::ZigLdArgs),

    /// Internal: Zig dlltool wrapper (called by shims)
    #[command(hide = true)]
    ZigDlltool(zig::tools::ZigDlltoolArgs),
}

#[derive(Subcommand)]
enum CacheCommands {
    /// Clear the cache directory
    Clear,

    /// Show cache directory location
    Path,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbose flag or RUST_LOG env var
    let verbose = matches!(&cli.command, Commands::Build(config) if config.verbose);
    setup_logging(verbose);

    match cli.command {
        Commands::Build(config) => {
            build::build(&config)?;
        }

        Commands::List => {
            build::list_targets()?;
        }

        Commands::Tools => {
            let assets = AssetManager::new()?;
            let host = tool_helpers::current_host_platform();
            let tools = tool_helpers::tools_for_host(&assets);

            println!("📦 Embedded tools for host: {host}\n");
            if tools.is_empty() {
                println!("⚠️  No tools embedded for this host platform.");
                println!("\nThis build does not include embedded tooling.");
                println!("You'll need to provide Zig and libclang manually.");
            } else {
                for tool in tools {
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("Tool:     {}", tool.name);
                    println!("Version:  {}", tool.version);
                    println!("BLAKE3:   {}", tool.blake3);
                    println!("Path:     {}", tool.archive_path);
                    if let Some(notes) = &tool.notes {
                        println!("Notes:    {}", notes);
                    }
                }
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                println!(
                    "\n✨ These tools will be automatically extracted and used during builds."
                );
                println!(
                    "   Cache location: {}",
                    assets.cache_dir().join("tools").display()
                );
            }
        }

        Commands::TestSdk => {
            let assets = AssetManager::new()?;

            println!("🧪 Testing macOS SDK embedding...\n");

            // Test x86_64 macOS SDK
            println!("Testing x86_64-darwin macOS SDK:");
            let temp_dir = std::env::temp_dir().join("rb_sys_test_sdk_x86");
            std::fs::create_dir_all(&temp_dir)?;
            match assets.extract_macos_sdk("x86_64-darwin", &temp_dir) {
                Ok(Some(sdk_path)) => {
                    println!("✅ Found macOS SDK: {}", sdk_path.display());

                    // Check if it has the expected structure
                    let usr_include = sdk_path.join("usr/include");
                    if usr_include.exists() {
                        println!("✅ SDK has usr/include directory");
                    } else {
                        println!("❌ SDK missing usr/include directory");
                    }

                    let frameworks = sdk_path.join("System/Library/Frameworks");
                    if frameworks.exists() {
                        println!("✅ SDK has System/Library/Frameworks directory");
                    } else {
                        println!("❌ SDK missing System/Library/Frameworks directory");
                    }
                }
                Ok(None) => {
                    println!("❌ No macOS SDK found in embedded assets");
                }
                Err(e) => {
                    println!("❌ Error extracting macOS SDK: {}", e);
                }
            }

            // Test aarch64 macOS SDK
            println!("\nTesting aarch64-darwin macOS SDK:");
            let temp_dir_arm = std::env::temp_dir().join("rb_sys_test_sdk_arm");
            std::fs::create_dir_all(&temp_dir_arm)?;
            match assets.extract_macos_sdk("aarch64-darwin", &temp_dir_arm) {
                Ok(Some(sdk_path)) => {
                    println!("✅ Found macOS SDK: {}", sdk_path.display());
                }
                Ok(None) => {
                    println!("❌ No macOS SDK found in embedded assets");
                }
                Err(e) => {
                    println!("❌ Error extracting macOS SDK: {}", e);
                }
            }

            println!("\n🎉 macOS SDK embedding test complete!");
        }

        Commands::Cache { action } => match action {
            CacheCommands::Clear => {
                assets::clear_cache()?;
            }
            CacheCommands::Path => {
                let cache_dir = assets::get_cache_dir()?;
                println!("{}", cache_dir.display());
            }
        },

        // Zig wrapper commands (called by shims)
        Commands::ZigCc(args) => {
            let target = zig::target::RustTarget::parse(&args.target)?;
            let shim = zig::tools::ZigCc {
                target,
                is_cxx: false,
            };
            shim.run(args)?;
        }
        Commands::ZigCxx(args) => {
            let target = zig::target::RustTarget::parse(&args.target)?;
            let shim = zig::tools::ZigCc {
                target,
                is_cxx: true,
            };
            shim.run(args)?;
        }
        Commands::ZigAr(args) => {
            let shim = zig::tools::ZigAr;
            shim.run(args)?;
        }
        Commands::ZigLd(args) => {
            let target = zig::target::RustTarget::parse(&args.target)?;
            let link_mode = match target.os {
                zig::target::Os::Windows => zig::args::LinkMode::Driver,
                _ => zig::args::LinkMode::Direct,
            };
            let shim = zig::tools::ZigLd { target, link_mode };
            shim.run(args)?;
        }
        Commands::ZigDlltool(args) => {
            let target = zig::target::RustTarget::parse(&args.target)?;
            let shim = zig::tools::ZigDlltool { target };
            shim.run(args)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
