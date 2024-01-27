use crate::card::hand::{DealerHand, PlayerHand};
use crate::game::Game;

pub mod io;
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

pub struct Player {
    pub chips: u32,
    strategy: Box<dyn Strategy>,
}

impl Player {
    pub fn new(chips: u32, strategy: impl Strategy + 'static) -> Self {
        Self { chips, strategy: Box::new(strategy) }
    }

    pub fn place_bet_or_quit(&mut self, game: &Game) -> GameAction {
        let action = self.strategy.place_bet_or_quit(game, self.chips);
        if let GameAction::Bet(bet) = &action {
            self.chips -= bet;
        }
        action
    }

    pub fn surrender_early(&self, game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> bool {
        self.strategy.surrender_early(game, player_hand, dealer_hand)
    }

    pub fn offer_insurance(&self, max_bet: u32) -> u32 {
        self.strategy.offer_insurance(max_bet)
    }

    pub fn get_hand_action(&self, game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> HandAction {
        self.strategy.get_hand_action(game, player_hand, dealer_hand, self.chips)
    }

    pub fn wait(&self) {
        self.strategy.wait();
    }
}

/// Represents the entity playing the game
pub trait Strategy {

    /// Prompts the player to place a bet or quit
    fn place_bet_or_quit(&mut self, game: &Game, chips: u32) -> GameAction;

    /// Prompts the player to surrender early or not
    /// Returns true if the player surrenders
    fn surrender_early(&self, game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> bool;

    /// Prompts the player to take insurance or not
    /// Returns the number of chips bet on insurance (0 if the player declines)
    fn offer_insurance(&self, max_bet: u32) -> u32;

    /// Prompts the player to make a move
    /// Which actions are available depends on the number of cards in the hand,
    /// whether the hand is a pair, and whether the player has enough chips to double their bet.
    /// Returns the action the player takes
    fn get_hand_action(&self, game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand, chips: u32) -> HandAction;

    /// Called for delays between actions
    /// Simulations can ignore this
    fn wait(&self) {}

}