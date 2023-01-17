use structopt::StructOpt;

use crate::commands::Command;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub command: Command,
}
