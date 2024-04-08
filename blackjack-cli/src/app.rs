use blackjack_core::blackjack::Table;
use blackjack_core::state::{GameState};

pub struct CliGame {
    pub table: Table,
    pub state: GameState,
}

impl CliGame {
    pub fn play(mut self) {
        println!("Welcome to Blackjack!");
        loop {
            let input = self.get_basic_strategy_input();
            match self.table.play(self.state, input) {
                Ok(new_state) => {
                    self.state = new_state;
                }
                Err(same_state) => {
                    self.state = same_state;
                }
            }
        }
    }
}


