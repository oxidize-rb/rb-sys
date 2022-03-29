mod command;

use atty::Stream;
use clap::Parser;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

trait CommandExecute {
    fn execute(self) -> eyre::Result<()>;
}

/// `cargo` stub for `cargo-rbx` (you probably meant to run `cargo rbx`)
#[derive(clap::Parser, Debug)]
#[clap(name = "cargo", bin_name = "cargo", version, propagate_version = true)]
struct CargoCommand {
    #[clap(subcommand)]
    subcommand: CargoSubcommands,
    /// Enable info logs, -vv for debug, -vvv for trace
    #[clap(short = 'v', long, parse(from_occurrences), global = true)]
    verbose: usize,
}

impl CommandExecute for CargoCommand {
    fn execute(self) -> eyre::Result<()> {
        self.subcommand.execute()
    }
}

#[derive(clap::Subcommand, Debug)]
enum CargoSubcommands {
    Rbx(command::rbx::Rbx),
    New(command::new::New),
}

impl CommandExecute for CargoSubcommands {
    fn execute(self) -> eyre::Result<()> {
        use CargoSubcommands::*;
        match self {
            Rbx(c) => c.execute(),
            New(c) => c.execute(),
        }
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::config::HookBuilder::default()
        .theme(if !atty::is(Stream::Stderr) {
            color_eyre::config::Theme::new()
        } else {
            color_eyre::config::Theme::default()
        })
        .install()?;

    let cargo_cli = CargoCommand::parse();

    // Initialize tracing with tracing-error, and eyre
    let fmt_layer = tracing_subscriber::fmt::Layer::new()
        .with_ansi(atty::is(Stream::Stderr))
        .with_writer(std::io::stderr)
        .pretty();

    let filter_layer = match EnvFilter::try_from_default_env() {
        Ok(filter_layer) => filter_layer,
        Err(_) => {
            let log_level = match cargo_cli.verbose {
                0 => "info",
                1 => "debug",
                _ => "trace",
            };
            let filter_layer = EnvFilter::new("warn");
            let filter_layer =
                filter_layer.add_directive(format!("cargo_rbx={}", log_level).parse()?);
            let filter_layer = filter_layer.add_directive(format!("rbx={}", log_level).parse()?);
            let filter_layer =
                filter_layer.add_directive(format!("rbx_macros={}", log_level).parse()?);
            let filter_layer =
                filter_layer.add_directive(format!("rbx_tests={}", log_level).parse()?);
            let filter_layer =
                filter_layer.add_directive(format!("rbx_pg_sys={}", log_level).parse()?);
            let filter_layer =
                filter_layer.add_directive(format!("rbx_utils={}", log_level).parse()?);
            filter_layer
        }
    };

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    cargo_cli.execute()
}
