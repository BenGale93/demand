use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    name: String,
    cost: u64,
    minimum: u64,
    count: u64,
}

impl Item {
    pub const fn new(name: String, cost: u64, minimum: u64, count: u64) -> Self {
        Self {
            name,
            cost,
            minimum,
            count,
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn cost(&self) -> u64 {
        self.cost
    }

    pub const fn cost_mut(&mut self) -> &mut u64 {
        &mut self.cost
    }

    pub const fn minimum(&self) -> u64 {
        self.minimum
    }

    pub const fn count(&self) -> u64 {
        self.count
    }

    pub const fn count_mut(&mut self) -> &mut u64 {
        &mut self.count
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: £{:.2}", self.name, self.cost as f64 / 100.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Buy {
    pub name: String,
    pub cost: u64,
}
