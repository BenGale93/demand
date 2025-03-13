use crate::prelude::*;

pub trait Pricer: Send {
    fn price(&mut self, items: &mut [Item]);

    fn update(&mut self, buys: &mut Vec<Buy>);
}

pub struct SimplePricer {
    last_buy: Option<String>,
}

impl SimplePricer {
    pub const fn new() -> Self {
        Self { last_buy: None }
    }
}

impl Default for SimplePricer {
    fn default() -> Self {
        Self::new()
    }
}

impl Pricer for SimplePricer {
    fn update(&mut self, buys: &mut Vec<Buy>) {
        if !buys.is_empty() {
            self.last_buy = buys.pop().map(|b| b.name);
            buys.clear();
        }
    }

    fn price(&mut self, items: &mut [Item]) {
        if let Some(last_buy) = &self.last_buy {
            for item in items {
                if item.name() == last_buy {
                    *item.cost_mut() += 1;
                } else {
                    *item.cost_mut() -= 1;
                }
            }
        }
    }
}
