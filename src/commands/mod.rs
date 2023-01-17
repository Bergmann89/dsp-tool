pub mod create_production_graph;

use structopt::StructOpt;

pub use create_production_graph::CreateProductionGraph;

use crate::error::Error;

#[derive(Debug, StructOpt)]
pub enum Command {
    CreateProductionGraph(CreateProductionGraph),
}

impl Command {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Self::CreateProductionGraph(cmd) => cmd.exec(),
        }
    }
}
