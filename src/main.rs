#![warn(clippy::all, clippy::nursery)]

use std::{collections::VecDeque, fs};

use serde::{Deserialize, Serialize};

pub mod error;
pub mod models;
pub mod pricing;

pub mod prelude {
    pub use crate::{error::InternalError, models::*};

    pub type Result<T> = core::result::Result<T, InternalError>;
}

use crate::{
    prelude::*,
    pricing::{Pricer, SimplePricer},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    items: Vec<Item>,
    buys: VecDeque<Buy>,
}

fn main() -> anyhow::Result<()> {
    let config_content = fs::read_to_string("config.toml")?;
    let mut config: Config = toml::from_str(&config_content)?;

    let mut pricer = SimplePricer::new(&config.items);

    while let Some(buy) = config.buys.pop_front() {
        pricer.update(vec![buy]);
        pricer.price(&mut config.items);
        for item in &mut config.items {
            println!("{}", item);
        }
    }
    Ok(())
}
