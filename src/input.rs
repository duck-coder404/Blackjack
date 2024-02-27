use std::thread;
use std::time::Duration;
use crate::card::hand::{DealerHand, PlayerHand};
use crate::game::Game;

pub mod cli;
pub mod basic;

pub enum GameAction {
    Bet(u32),
    Quit,
}

/// The final action taken by the player
pub enum HandAction {
    Stand,
    Hit,
    Double,
    Split,
    Surrender,
}

/// Represents the entity playing the game
pub enum Input {
    Basic {
        turns: u32,
        flat_bet: u32,
    },
    CLI,
}

pub struct Player {
    pub chips: u32,
    strategy: Input,
}

impl Player {
    pub fn new(chips: u32, strategy: Input) -> Self {
        Self { chips, strategy }
    }

    /// Prompts the player to place a bet or quit
    pub fn place_bet_or_quit(&mut self, game: &Game) -> GameAction {
        match self.strategy {
            Input::Basic { mut turns, flat_bet } => basic::place_bet_or_quit(game, self.chips, &mut turns, flat_bet),
            Input::CLI => cli::place_bet_or_quit(game, self.chips),
        }
    }

    /// Prompts the player to surrender early or not
    /// Returns true if the player surrenders
    pub fn surrender_early(&self, game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> bool {
        match self.strategy {
            Input::Basic { .. } => basic::surrender_early(game, player_hand, dealer_hand),
            Input::CLI => cli::surrender_early(game, player_hand, dealer_hand),
        }
    }
    
    /// Prompts the player to take insurance or not
    /// Returns the number of chips bet on insurance (0 if the player declines)
    pub fn offer_insurance(&self, max_bet: u32) -> u32 {
        match self.strategy {
            Input::Basic { .. } => basic::offer_insurance(max_bet),
            Input::CLI => cli::offer_insurance(max_bet),
        }
    }

    /// Prompts the player to make a move
    /// Which actions are available depends on the number of cards in the hand,
    /// whether the hand is a pair, and whether the player has enough chips to double their bet.
    /// Returns the action the player takes
    pub fn get_hand_action(&self, game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> HandAction {
        match self.strategy {
            Input::Basic { .. } => basic::get_hand_action(game, player_hand, dealer_hand, self.chips),
            Input::CLI => cli::get_hand_action(game, player_hand, dealer_hand, self.chips),
        }
    }

    pub fn wait(&self) {
        match self.strategy {
            Input::Basic { .. } => {}
            Input::CLI => thread::sleep(Duration::from_secs(1)),
        }
    }
}
