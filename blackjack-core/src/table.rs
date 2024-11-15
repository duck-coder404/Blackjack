use async_trait::async_trait;
use crate::card::Card;
use crate::card::hand::{DealerHand, PlayerHand};
use crate::card::shoe::Shoe;
use crate::game::Input;
use crate::rules::Rules;
use crate::statistics::Statistics;

#[async_trait]
pub trait Player {
    /// Get the player to bet, or decide on surrender, or play a hand.
    async fn get_input(&mut self) -> Input;
    fn chips(&self) -> u32;
    fn chips_mut(&mut self) -> &mut u32;
}

pub trait Dispenser {
    fn deal(&mut self) -> Card;
    fn needs_shuffle(&self) -> bool;
    fn shuffle(&mut self);
}

impl Dispenser for Shoe {
    fn deal(&mut self) -> Card {
        self.draw_card()
    }
    fn needs_shuffle(&self) -> bool {
        self.needs_shuffle()
    }
    fn shuffle(&mut self) {
        self.shuffle()
    }
}

pub struct NewTable {
    dispenser: Box<dyn Dispenser>,
    rules: Rules,
    statistics: Statistics,
}

pub struct Round<'table> {
    table: &'table mut NewTable,
    state: GameState,
}

fn main() {
    let table = NewTable {
        dispenser: Box::new(Shoe::new(0, 0.0)),
        rules: Rules::default(),
        statistics: Statistics::default(),
    };
}