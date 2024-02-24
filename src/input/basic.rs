use crate::card::hand::{DealerHand, PlayerHand};
use crate::game::Game;
use crate::input::{HandAction, GameAction, Strategy};

pub struct BasicStrategy {
    turns: u32,
    flat_bet: u32,
}

impl BasicStrategy {
    pub fn new(turns: u32, flat_bet: u32) -> Self {
        Self {
            turns,
            flat_bet,
        }
    }
}

impl Strategy for BasicStrategy {
    fn place_bet_or_quit(&mut self, _: &Game, _: u32) -> GameAction {
        if self.turns == 0 { GameAction::Quit } else {
            self.turns -= 1;
            GameAction::Bet(self.flat_bet)
        }
    }

    fn surrender_early(&self, game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> bool {
        match (player_hand.value.soft, player_hand.is_pair()) {
            (false, false) => surrender_early_hard(game, player_hand, dealer_hand),
            (true, false) => false,
            (_, true) => surrender_early_pair(game, player_hand, dealer_hand),
        }
    }

    fn offer_insurance(&self, _: u32) -> u32 {
        0
    }

    fn get_hand_action(&self, game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand, chips: u32) -> HandAction {
        let preferred = match (player_hand.value.soft, player_hand.is_pair()) {
            (false, false) => make_move_hard(game, player_hand, dealer_hand),
            (true, false) => make_move_soft(game, player_hand, dealer_hand),
            (_, true) => make_move_pair(game, player_hand, dealer_hand),
        };
        let can_double_chips = chips >= player_hand.bet;
        let two_cards = player_hand.cards.len() == 2;
        let can_double_after_split = player_hand.splits == 0 || game.double_after_split;
        preferred.resolve(
            can_double_chips && two_cards && can_double_after_split,
            game.late_surrender,
            game.double_after_split && can_double_chips,
        )
    }

}

/// Source: <https://wizardofodds.com/games/blackjack/surrender/>
fn surrender_early_hard(game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> bool {
    match (player_hand.value.total, dealer_hand.showing()) {
        (14, 10) if game.dispenser.decks <= 2 && player_hand.composed_of(10, 4) => false,
        (14, 10) if game.dispenser.decks == 1 && player_hand.composed_of(5, 9) => false,
        (5..=7 | 12..=17, 11) | (14..=16, 10) => true,
        _ => false
    }
}

/// Source: <https://wizardofodds.com/games/blackjack/surrender/>
fn surrender_early_pair(game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> bool {
    match (player_hand.cards[0].value().total, dealer_hand.showing()) {
        (8, 10) if game.dispenser.decks == 1 && game.double_after_split => false,
        (7..=8, 10) | (3 | 6..=8, 11) => true,
        (2, 11) if game.soft_17_hit => true,
        _ => false
    }
}

/// The preferred action which may involve a fallback action
enum PreferredAction {
    Stand,
    Hit,
    Split,
    DoubleOrHit,
    DoubleOrStand,
    SurrenderOrHit,
    SplitOrHit,
}

impl PreferredAction {
    /// Converts the preferred action to an action given the current game situation
    pub fn resolve(self, can_double: bool, can_surrender: bool, can_split: bool) -> HandAction {
        match self {
            PreferredAction::Stand => HandAction::Stand,
            PreferredAction::Hit => HandAction::Hit,
            PreferredAction::Split => HandAction::Split,
            PreferredAction::DoubleOrHit => if can_double { HandAction::Double } else { HandAction::Hit },
            PreferredAction::DoubleOrStand => if can_double { HandAction::Double } else { HandAction::Stand },
            PreferredAction::SurrenderOrHit => if can_surrender { HandAction::Surrender } else { HandAction::Hit },
            PreferredAction::SplitOrHit => if can_split { HandAction::Split } else { HandAction::Hit },
        }
    }
}

fn make_move_hard(game: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> PreferredAction {
    match (player_hand.value.total, dealer_hand.showing()) {
        (9, 2) if game.dispenser.decks <= 2 => PreferredAction::DoubleOrHit,
        (9, 3..=6) => PreferredAction::DoubleOrHit,
        (10, 2..=9) => PreferredAction::DoubleOrHit,
        (11, 2..=10) => PreferredAction::DoubleOrHit,
        (11, 11) if game.dispenser.decks <= 2 => PreferredAction::DoubleOrHit,
        (12, 2..=3) => PreferredAction::Hit,
        (15, 10) if game.dispenser.decks >= 8 => PreferredAction::SurrenderOrHit,
        (16, 9) if game.dispenser.decks >= 4 => PreferredAction::SurrenderOrHit,
        (16, 10..=11) => PreferredAction::SurrenderOrHit,
        (4..=11, 2..=11) => PreferredAction::Hit,
        (12..=16, 2..=6) => PreferredAction::Stand,
        (12..=16, 7..=11) => PreferredAction::Hit,
        (17..=21, 2..=11) => PreferredAction::Stand,
        (_, showing) => panic!("Invalid hand value: {} against {}", player_hand.value, showing),
    }
}

fn make_move_soft(_: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> PreferredAction {
    match (player_hand.value.total, dealer_hand.showing()) {
        (13..=14, 5..=6) => PreferredAction::DoubleOrHit,
        (15..=16, 4..=6) => PreferredAction::DoubleOrHit,
        (17, 3..=6) => PreferredAction::DoubleOrHit,
        (18, 3..=6) => PreferredAction::DoubleOrStand,
        (18, 2) | (18, 7..=8) => PreferredAction::Stand,
        (13..=18, 2..=11) => PreferredAction::Hit,
        (19..=21, 2..=11) => PreferredAction::Stand,
        (_, showing) => panic!("Invalid hand value: {} against {}", player_hand.value, showing),
    }
}

fn make_move_pair(_: &Game, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> PreferredAction {
    match (player_hand.cards[0].value().total, dealer_hand.showing()) {
        (2..=3, 2..=3) => PreferredAction::SplitOrHit,
        (2..=3, 4..=7) => PreferredAction::Split,
        (4, 5..=6) => PreferredAction::SplitOrHit,
        (5, 2..=9) => PreferredAction::DoubleOrHit,
        (6, 2) => PreferredAction::SplitOrHit,
        (6, 3..=6) => PreferredAction::Split,
        (7, 2..=7) => PreferredAction::Split,
        (2..=7, 2..=11) => PreferredAction::Hit,
        (9, 7 | 10..=11) | (10, 2..=11) => PreferredAction::Stand,
        (8..=11, 2..=11) => PreferredAction::Split,
        (_, showing) => panic!("Invalid hand value: {} against {}", player_hand.value, showing),
    }
}
