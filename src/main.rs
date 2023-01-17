use log::LevelFilter;

use dsp_tool::{args::Args, error::Error};
use structopt::StructOpt;

fn main() -> Result<(), Error> {
    env_logger::Builder::from_default_env()
        .format_level(true)
        .format_module_path(false)
        .format_target(false)
        .format_timestamp(None)
        .filter_level(LevelFilter::Info)
        .init();

    let Args { command } = Args::from_args();

    if let Err(err) = command.exec() {
        log::error!("Error while executing the command: {}", err);
    }

    Ok(())
}
