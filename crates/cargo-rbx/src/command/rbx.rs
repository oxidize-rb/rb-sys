use crate::CommandExecute;

#[derive(clap::Args, Debug)]
#[clap(about, author)]
pub(crate) struct Rbx {
    #[clap(subcommand)]
    subcommand: CargoRbxSubCommands,
    #[clap(from_global, parse(from_occurrences))]
    verbose: usize,
}

impl CommandExecute for Rbx {
    fn execute(self) -> eyre::Result<()> {
        self.subcommand.execute()
    }
}

#[derive(clap::Subcommand, Debug)]
enum CargoRbxSubCommands {
    New(super::new::New),
}

impl CommandExecute for CargoRbxSubCommands {
    fn execute(self) -> eyre::Result<()> {
        use CargoRbxSubCommands::*;
        match self {
            New(c) => c.execute(),
        }
    }
}
