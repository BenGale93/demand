use crate::prelude::*;

pub trait Pricer {
    fn new(items: &[Item]) -> Self;

    fn price(&mut self, items: &mut [Item]);

    fn update(&mut self, buys: Vec<Buy>);
}

pub struct SimplePricer {
    last_buy: Option<String>,
}

impl Pricer for SimplePricer {
    fn new(_items: &[Item]) -> Self {
        Self { last_buy: None }
    }

    fn update(&mut self, mut buys: Vec<Buy>) {
        self.last_buy = buys.pop().map(|b| b.name);
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
