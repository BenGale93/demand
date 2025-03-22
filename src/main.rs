#![warn(clippy::all, clippy::nursery)]

use clap::{Parser, Subcommand};
use connect::{connect_to_server, view_prices};
use serde::{Deserialize, Serialize};
use server::pricer_server;

pub mod commands;
pub mod connect;
pub mod error;
pub mod models;
pub mod pricing;
pub mod server;

pub mod prelude {
    pub use crate::{error::InternalError, models::*};

    pub type Result<T> = core::result::Result<T, InternalError>;
}

use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    items: Vec<Item>,
}

#[derive(Parser)]
#[command(about = "Interact with the demand pricer.")]
pub struct DemandCli {
    #[command(subcommand)]
    pub command: DemandCommands,
}

#[derive(Subcommand)]
pub enum DemandCommands {
    /// Start the pricer server.
    Start,
    /// Connect to the pricer server.
    Connect,
    /// View the prices.
    View,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = DemandCli::parse();
    let port = "8080";
    match cli.command {
        DemandCommands::Start => pricer_server(port).await?,
        DemandCommands::Connect => connect_to_server(port).await,
        DemandCommands::View => view_prices(port).await,
    }
    Ok(())
}
