#![feature(vec_pop_if)]
#![warn(clippy::result_large_err)]

//! The core logic of the game.

use std::fmt;

use crate::card::hand::{DealerHand, PlayerHand, ActiveTurn, Status, PendingTurn, FinishedTurn};
use crate::card::shoe::Shoe;
use crate::rules::Rules;
use crate::state::GameState;
use crate::statistics::Statistics;

/// The player's options for playing their hand
#[derive(Debug, PartialEq, Eq)]
pub enum HandAction {
    Stand,
    Hit,
    Double,
    Split,
    Surrender,
}

/// The game input. Different states require different inputs.
#[derive(Debug)]
pub enum Input {
    Bet(u32),
    Choice(bool),
    Action(HandAction),
}

/// The game table. This is where the game is played.
/// It holds the shoe, and the game rules.
#[derive(Debug)]
pub struct Table {
    pub shoe: Shoe,             // The shoe of cards used in the game
    pub rules: Rules,           // The table rules
    pub statistics: Statistics, // The game statistics
    pub fast_forward: bool,     // Fast-forward non-user-facing transitions and skip input checks for faster simulation
}

// TODO: The CandAfford variants of these errors should be handled elsewhere.
// the player shouldn't be able to bet more than they have in the first place.
#[derive(Debug, PartialEq, Eq)]
pub enum BetError {
    TooLow,
    TooHigh,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DoubleError {
    NotTwoCards,
    DoubleAfterSplitNotAllowed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SplitError {
    NotAPair,
    MaxSplitsReached,
    SplitAcesNotAllowed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SurrenderError {
    NotTwoCards,
    LateSurrenderNotAllowed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    WrongInput,
    BetError(BetError),
    DoubleError(DoubleError),
    SplitError(SplitError),
    SurrenderError(SurrenderError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongInput => write!(f, "Wrong input"),
            Self::BetError(err) => match err {
                BetError::TooLow => write!(f, "Bet too low"),
                BetError::TooHigh => write!(f, "Bet too high"),
            },
            Self::DoubleError(err) => match err {
                DoubleError::NotTwoCards => write!(f, "Not two cards"),
                DoubleError::DoubleAfterSplitNotAllowed => {
                    write!(f, "Double after split not allowed")
                }
            },
            Self::SplitError(err) => match err {
                SplitError::NotAPair => write!(f, "Not a pair"),
                SplitError::MaxSplitsReached => write!(f, "Max splits reached"),
                SplitError::SplitAcesNotAllowed => write!(f, "Split aces not allowed"),
            },
            Self::SurrenderError(err) => match err {
                SurrenderError::NotTwoCards => write!(f, "Not two cards"),
                SurrenderError::LateSurrenderNotAllowed => write!(f, "Late surrender not allowed"),
            },
        }
    }
}

/// If the player input is invalid, the game cannot progress.
/// In these cases, the game returns an error with the unchanged state and the reason for the error.
pub type ProgressResult = Result<GameState, (GameState, Error)>;

impl Table {
    #[must_use]
    pub const fn new(shoe: Shoe, rules: Rules) -> Self {
        Self {
            shoe,
            rules,
            statistics: Statistics::new(),
            fast_forward: false,
        }
    }

    /// Plays the game from the given state and input.
    /// Returns the next state of the game, or the same state if the game could not progress.
    /// # Errors
    /// Returns Err with the same state if the game could not progress.
    #[rustfmt::skip]
    pub fn progress(&mut self, state: GameState, input: Option<Input>) -> ProgressResult {
        match state {
            GameState::Betting => {
                if let Some(Input::Bet(bet)) = input {
                    self.bet(bet)
                } else {
                    Err((GameState::Betting, Error::WrongInput))
                }
            },
            GameState::DealFirstPlayerCards { bets, player_turns } => {
                Ok(self.deal_first_player_card(bets, player_turns))
            },
            GameState::DealFirstDealerCard { player_turns } => {
                Ok(self.deal_first_dealer_card(player_turns))
            },
            GameState::DealSecondPlayerCards { player_turns, dealer_hand } => {
                Ok(self.deal_second_player_card(player_turns, dealer_hand))
            },
            GameState::DealHoleCard { player_turns, dealer_hand } => {
                Ok(self.deal_hole_card(player_turns, dealer_hand))
            },
            GameState::OfferEarlySurrender { player_turns, dealer_hand } => {
                if let Some(Input::Choice(early_surrender)) = input {
                    Ok(self.choose_early_surrender(player_turns, dealer_hand, vec![early_surrender]))
                } else {
                    Err((
                        GameState::OfferEarlySurrender {
                            player_turns,
                            dealer_hand,
                        },
                        Error::WrongInput,
                    ))
                }
            }
            GameState::OfferInsurance { player_turns, dealer_hand } => {
                if let Some(Input::Bet(insurance_bet)) = input {
                    Ok(self.bet_insurance(player_turns, dealer_hand, vec![insurance_bet]))
                } else {
                    Err((
                        GameState::OfferInsurance {
                            player_turns,
                            dealer_hand,
                        },
                        Error::WrongInput,
                    ))
                }
            }
            GameState::CheckDealerHoleCard { player_turns, dealer_hand } => {
                Ok(self.check_dealer_hole_card(player_turns, dealer_hand))
            },
            GameState::PlayPlayerTurn { pending_turns, current_turn, finished_turns, dealer_hand } => {
                if let Some(Input::Action(action)) = input {
                    self.play_player_turn(pending_turns, current_turn, finished_turns, dealer_hand, action)
                } else {
                    Err((
                        GameState::PlayPlayerTurn {
                            pending_turns,
                            current_turn,
                            finished_turns,
                            dealer_hand,
                        },
                        Error::WrongInput,
                    ))
                }
            }
            GameState::PlayerStand { pending_turns, current_turn, finished_turns, dealer_hand } => {
                Ok(self.stand(pending_turns, current_turn, finished_turns, dealer_hand))
            },
            GameState::PlayerHit { pending_turns, current_turn, finished_turns, dealer_hand } => {
                Ok(self.hit(pending_turns, current_turn, finished_turns, dealer_hand))
            },
            GameState::PlayerDouble { pending_turns, current_turn, finished_turns, dealer_hand } => {
                Ok(self.double(pending_turns, current_turn, finished_turns, dealer_hand))
            },
            GameState::PlayerSplit { pending_turns, current_turn, finished_turns, dealer_hand } => {
                Ok(self.split(pending_turns, current_turn, finished_turns, dealer_hand))
            },
            GameState::DealFirstSplitCard { pending_turns, current_turn, new_hand, finished_turns, dealer_hand } => {
                Ok(self.deal_first_split_card(pending_turns, current_turn, new_hand, finished_turns, dealer_hand))
            },
            GameState::DealSecondSplitCard { pending_turns, current_turn, new_hand, finished_turns, dealer_hand } => {
                Ok(self.deal_second_split_card(pending_turns, current_turn, new_hand, finished_turns, dealer_hand))
            },
            GameState::PlayerSurrender { pending_turns, current_turn, finished_turns, dealer_hand } => {
                Ok(self.surrender(pending_turns, current_turn, finished_turns, dealer_hand))
            },
            GameState::RevealHoleCard { finished_turns, dealer_hand } => {
                Ok(self.play_dealer_turn_or_end_round(finished_turns, dealer_hand))
            },
            GameState::PlayDealerTurn { finished_turns, dealer_hand } => {
                Ok(self.play_dealer_turn(finished_turns, dealer_hand))
            },
            GameState::RoundOver { finished_turns, dealer_hand } => {
                Ok(self.end_round(finished_turns, dealer_hand))
            },
            GameState::Payout { total_bets, .. } => {
                Ok(self.pay_out_winnings(total_bets))
            }
            GameState::Shuffle => Ok(self.shuffle_dispenser()),
        }
    }

    /// A helper function to determine if the player is allowed to double down on their current hand.
    /// The player can double down if their hand consists of two cards, they have enough chips,
    /// and the maximum splits and double after split rules do not prevent them from doing so.
    /// # Errors
    /// Returns an error containing the reason why the player cannot double down.
    pub fn check_double_allowed(&self, player_turn: &ActiveTurn) -> Result<(), DoubleError> {
        if player_turn.current_hand().size() != 2 {
            Err(DoubleError::NotTwoCards)
        } else if player_turn.hands() > 1 && !self.rules.double_after_split {
            Err(DoubleError::DoubleAfterSplitNotAllowed)
        } else {
            Ok(())
        }
    }

    /// A helper function to determine if the player is allowed to split their current hand.
    /// The player can split if their hand is a pair, they have enough chips to double their bet,
    /// and the maximum splits and split-aces rules do not prevent them from doing so.
    /// # Errors
    /// Returns an error containing the reason why the player cannot split.
    pub fn check_split_allowed(&self, player_turn: &ActiveTurn) -> Result<(), SplitError> {
        if !player_turn.current_hand().is_pair() {
            Err(SplitError::NotAPair)
        } else if player_turn.current_hand().value.soft && !self.rules.split_aces {
            Err(SplitError::SplitAcesNotAllowed)
        } else if self
            .rules
            .max_splits
            .map_or(false, |max| player_turn.hands() > max)
        {
            Err(SplitError::MaxSplitsReached)
        } else {
            Ok(())
        }
    }

    /// A helper function to determine if the player is allowed to surrender their current hand.
    /// The player can surrender if their hand consists of two cards and late surrendering
    /// is enabled in the game configuration.
    /// # Errors
    /// Returns an error containing the reason why the player cannot surrender.
    pub fn check_surrender_allowed(&self, hand: &PlayerHand) -> Result<(), SurrenderError> {
        if hand.size() != 2 {
            Err(SurrenderError::NotTwoCards)
        } else if !self.rules.late_surrender {
            Err(SurrenderError::LateSurrenderNotAllowed)
        } else {
            Ok(())
        }
    }

    /// The player places a bet to start the round.
    /// The bet must be within the table limits and the player must have enough chips.
    /// If the bet is valid, the game transitions to dealing the first player card.
    fn bet(&mut self, bet: u32) -> ProgressResult {
        if self.fast_forward {
            self.chips -= bet;
            // Simulated bets should already be valid, so we don't need to check them
            return Ok(self.deal_first_player_card(bet));
        }
        match (self.rules.min_bet, self.rules.max_bet) {
            (Some(min), _) if bet < min => {
                Err((GameState::Betting, Error::BetError(BetError::TooLow)))
            }
            (_, Some(max)) if bet > max => {
                Err((GameState::Betting, Error::BetError(BetError::TooHigh)))
            }
            _ if bet > self.chips => {
                Err((GameState::Betting, Error::BetError(BetError::CantAfford)))
            }
            _ => {
                self.chips -= bet;
                Ok(GameState::DealFirstPlayerCards { bets: vec![bet], player_turns: vec![] })
            }
        }
    }

    /// The dealer deals the first card to the player and the player's hand is created.
    /// Next, the dealer will deal their first card.
    fn deal_first_player_card(
        &mut self,
        mut bets: Vec<u32>,
        mut player_turns: Vec<PendingTurn>,
    ) -> GameState {
        // If there is another bet, draw a card for the player and create a new hand
        if let Some(bet) = bets.pop() {
            let card = self.shoe.draw_card();
            player_turns.push(PlayerHand::new(card, bet).into());
        }
        if bets.is_empty() {
            // If there are no more bets, the dealer will deal their first card
            if self.fast_forward {
                self.deal_first_dealer_card(player_turns)
            } else {
                GameState::DealFirstDealerCard { player_turns }
            }
        } else {
            // If there are more bets, continue dealing cards to the players
            if self.fast_forward {
                self.deal_first_player_card(bets, player_turns)
            } else {
                GameState::DealFirstPlayerCards { bets, player_turns }
            }
        }
    }

    /// The dealer deals the first card to themselves and the dealer's hand is created.
    /// Next, the dealer will deal the second card to the player.
    fn deal_first_dealer_card(
        &mut self,
        player_turns: Vec<PendingTurn>
    ) -> GameState {
        let card = self.shoe.draw_card();
        let dealer_hand = DealerHand::new(card, self.rules.dealer_soft_17);
        if self.fast_forward {
            self.deal_second_player_card(player_turns, dealer_hand)
        } else {
            GameState::DealSecondPlayerCards {
                player_turns,
                dealer_hand,
            }
        }
    }

    /// The dealer deals the second card to the player.
    /// Next, the dealer will deal the second card to themselves, also known as the hole card.
    fn deal_second_player_card(
        &mut self,
        mut player_turns: Vec<PendingTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        // If there is another hand which needs another card, draw a card for the player
        if let Some(hand) = player_turns
            .iter_mut()
            .find(|hand| hand.size() == 1) {
            *hand += self.shoe.draw_card();
            if self.fast_forward {
                self.deal_second_player_card(player_turns, dealer_hand)
            } else {
                GameState::DealSecondPlayerCards { player_turns, dealer_hand }
            }
        } else {
            if self.fast_forward {
                self.deal_hole_card(player_turns, dealer_hand)
            } else {
                GameState::DealHoleCard { player_turns, dealer_hand }
            }
        }
    }

    /// The dealer deals the hole card to themselves.
    /// If the dealer is showing a 10 or an Ace, they will check their hole card for Blackjack.
    /// If early surrender or insurance is offered, the game will transition to those states first.
    /// If the dealer cannot have Blackjack or if the player does have Blackjack, the dealer will
    /// not check their hole card, and will instead let the player play their hand.
    fn deal_hole_card(
        &mut self,
        mut player_turns: Vec<PendingTurn>,
        mut dealer_hand: DealerHand,
    ) -> GameState {
        dealer_hand += self.shoe.draw_card();
        if dealer_hand.showing() < 10 || player_turns.iter().all(|turn| turn.hand.status == Status::Blackjack) {
            // The dealer cannot have Blackjack or all players have Blackjack,
            // so the dealer will not check their hole card or offer early surrender or insurance
            self.start_player_phase_or_end_round(player_turns, dealer_hand)
        } else if self.rules.early_surrender {
            // The dealer is showing a 10 or greater and early surrender is offered
            // This will give players a chance to surrender before the dealer checks for Blackjack
            GameState::OfferEarlySurrender {
                player_turns,
                dealer_hand,
            }
        } else if self.rules.insurance && dealer_hand.showing() == 11 {
            // The dealer is showing an ace and insurance is offered
            // This will give players a chance to place an insurance bet before the dealer checks for Blackjack
            GameState::OfferInsurance {
                player_turns,
                dealer_hand,
            }
        } else {
            // The dealer is showing at least a 10 and no early surrender or insurance is offered
            // The dealer checks their hole card for Blackjack
            if self.fast_forward {
                self.check_dealer_hole_card(player_turns, dealer_hand)
            } else {
                GameState::CheckDealerHoleCard {
                    player_turns,
                    dealer_hand,
                }
            }
        }
    }

    /// The player decides whether to surrender early.
    /// If the player surrenders, their hand is finished and the round is over.
    /// Otherwise, if insurance is offered and the dealer is showing an Ace, the player can place
    /// an insurance bet.
    /// Otherwise, the dealer checks their hole card for Blackjack.
    fn choose_early_surrender(
        &mut self,
        mut player_turns: Vec<PendingTurn>,
        dealer_hand: DealerHand,
        surrender_choices: Vec<bool>,
    ) -> GameState {
        assert_eq!(player_turns.len(), surrender_choices.len()); // There should be a surrender decision for each player hand
        for (turn, &should_surrender) in player_turns.iter_mut().zip(&surrender_choices) {
            if should_surrender {
                turn.hand.surrender();
            }
        }
        if surrender_choices.iter().all(|&surrender| surrender) {
            // All players have surrendered, so the round is over
            let finished_turns = player_turns.into_iter().map(|turn| turn.into()).collect();
            if self.fast_forward {
                self.end_round(finished_turns, dealer_hand)
            } else {
                GameState::RoundOver {
                    finished_turns,
                    dealer_hand,
                }
            }
        } else if self.rules.insurance && dealer_hand.showing() == 11 {
            GameState::OfferInsurance {
                player_turns,
                dealer_hand,
            }
        } else {
            if self.fast_forward {
                self.check_dealer_hole_card(player_turns, dealer_hand)
            } else {
                GameState::CheckDealerHoleCard {
                    player_turns,
                    dealer_hand,
                }
            }
        }
    }

    /// The player places an insurance bet.
    /// The bet must be less than half of the player's original bet,
    /// and the player must have enough chips.
    /// Next, the dealer will check their hole card for Blackjack.
    fn bet_insurance(
        &mut self,
        mut player_turns: Vec<PendingTurn>,
        dealer_hand: DealerHand,
        insurance_bets: Vec<u32>,
    ) -> GameState {
        assert_eq!(player_turns.len(), insurance_bets.len()); // There should be an insurance bet for each player hand
        for (turn, insurance_bet) in player_turns.iter_mut().zip(insurance_bets) {
            // TODO: We should probably return an error if the bet is too large, but we don't have a way to handle it yet
            turn.insurance_bet = if insurance_bet > turn.hand.bet / 2 {
                turn.hand.bet / 2
            } else {
                insurance_bet
            };
        }
        if self.fast_forward {
            self.check_dealer_hole_card(player_turns, dealer_hand)
        } else {
            GameState::CheckDealerHoleCard {
                player_turns,
                dealer_hand,
            }
        }
    }

    /// The dealer checks their hole card for Blackjack.
    /// If the dealer does not have Blackjack, it is the player's turn to play their hand.
    /// If the dealer does have Blackjack, the round is over.
    fn check_dealer_hole_card(
        &mut self,
        mut player_turns: Vec<PendingTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        if dealer_hand.status == Status::Blackjack {
            // The dealer has Blackjack, so the round is over before the players play their hands
            let finished_turns = player_turns.into_iter().map(|turn| turn.into()).collect();
            if self.fast_forward {
                self.end_round(finished_turns, dealer_hand)
            } else {
                GameState::RoundOver {
                    finished_turns,
                    dealer_hand,
                }
            }
        } else {
            self.start_player_phase_or_end_round(player_turns, dealer_hand)
        }
    }
    
    /// The dealer has finished dealing, and it is the player's turn to play their hand.
    /// If there are players in the game, the first player will play their hand.
    /// If there are no players, the dealer will reveal their hole card and stand, ending the round.
    fn start_player_phase_or_end_round(
        &mut self,
        mut player_turns: Vec<PendingTurn>,
        mut dealer_hand: DealerHand,
    ) -> GameState {
        let finished_turns = Vec::with_capacity(player_turns.len());
        if let Some(first_turn) = player_turns.pop() {
            // If we have at least one player, the player will play their hand
            self.continue_player_phase_or_go_to_dealer(
                player_turns,
                first_turn.into(),
                finished_turns, // We will fill this with finished turns
                dealer_hand
            )
        } else {
            // If there are no players, the dealer flips their hole card and stands
            dealer_hand.status = Status::Stood;
            if self.fast_forward {
                self.end_round(finished_turns, dealer_hand)
            } else {
                GameState::RoundOver {
                    finished_turns,
                    dealer_hand,
                }
            }
        }
    }

    /// We check if the player still has a hand to play.
    /// If so, we continue the player's turn.
    /// Otherwise, the player's turn is over and the dealer will reveal their hole card.
    fn continue_player_phase_or_go_to_dealer(
        &mut self,
        mut pending_turns: Vec<PendingTurn>,
        current_turn: ActiveTurn,
        mut finished_turns: Vec<FinishedTurn>,
        mut dealer_hand: DealerHand,
    ) -> GameState {
        match current_turn.continue_playing() {
            // The current player's turn is not over
            Ok(current_turn) => GameState::PlayPlayerTurn {
                finished_turns,
                current_turn,
                pending_turns,
                dealer_hand,
            },
            // The current player's turn is over
            // Move to the next player if there is one, otherwise go to the dealer's turn
            Err(finished_turn) => {
                finished_turns.push(finished_turn);
                // Skip any hands that are no longer in play
                while let Some(turn) = pending_turns.pop_if(|turn| turn.hand.status != Status::InPlay) {
                    finished_turns.push(turn.into());
                }
                // If there are still hands in play, go to the next player's turn
                if let Some(next_player_hand) = pending_turns.pop() {
                    GameState::PlayPlayerTurn {
                        finished_turns,
                        current_turn: next_player_hand.into(),
                        pending_turns,
                        dealer_hand,
                    }
                } else {
                    // All players are done; go to dealer turn.
                    // If none of the players explicitly stood on any of their hands,
                    // the dealer will simply flip their hole card and stand immediately.
                    if dealer_hand.status == Status::InPlay && !finished_turns.iter()
                        .flat_map(|hands| hands.iter())
                        .any(|hand| hand.status == Status::Stood)
                    {
                        dealer_hand.status = Status::Stood;
                    }
                    if self.fast_forward {
                        self.play_dealer_turn_or_end_round(finished_turns, dealer_hand)
                    } else {
                        GameState::RevealHoleCard {
                            finished_turns,
                            dealer_hand,
                        }
                    }
                }
            }
        }
    }

    /// It is the player's turn.
    /// The player starts with one hand, but may have more if they split.
    /// The player can choose to stand, hit, double down, split, or surrender.
    /// Doubling down, splitting, and surrendering are only allowed in certain circumstances.
    fn play_player_turn(
        &mut self,
        pending_turns: Vec<PendingTurn>,
        current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
        action: HandAction,
    ) -> ProgressResult {
        match action {
            HandAction::Hit if self.fast_forward => {
                Ok(self.hit(pending_turns, current_turn, finished_turns, dealer_hand))
            }
            HandAction::Hit => Ok(GameState::PlayerHit {
                pending_turns,
                current_turn,
                finished_turns,
                dealer_hand,
            }),
            HandAction::Stand if self.fast_forward => {
                Ok(self.stand(pending_turns, current_turn, finished_turns, dealer_hand))
            }
            HandAction::Stand => Ok(GameState::PlayerStand {
                pending_turns,
                current_turn,
                finished_turns,
                dealer_hand,
            }),
            HandAction::Double if self.fast_forward => {
                // Simulated moves should already be valid, so we don't need to check them
                // self.chips -= current_turn.current_hand().bet; TODO: Figure out chip handling
                Ok(self.double(pending_turns, current_turn, finished_turns, dealer_hand))
            }
            HandAction::Double => {
                if let Err(err) = self.check_double_allowed(&current_turn) {
                    Err((
                        GameState::PlayPlayerTurn {
                            pending_turns,
                            current_turn,
                            finished_turns,
                            dealer_hand,
                        },
                        Error::DoubleError(err),
                    ))
                } else {
                    // self.chips -= current_turn.current_hand().bet; TODO
                    Ok(GameState::PlayerDouble {
                        pending_turns,
                        current_turn,
                        finished_turns,
                        dealer_hand,
                    })
                }
            }
            HandAction::Split if self.fast_forward => {
                // Simulated moves should already be valid, so we don't need to check them
                // self.chips -= current_turn.current_hand().bet; TODO
                Ok(self.split(pending_turns, current_turn, finished_turns, dealer_hand))
            }
            HandAction::Split => {
                if let Err(err) = self.check_split_allowed(&current_turn) {
                    Err((
                        GameState::PlayPlayerTurn {
                            pending_turns,
                            current_turn,
                            finished_turns,
                            dealer_hand,
                        },
                        Error::SplitError(err),
                    ))
                } else {
                    // self.chips -= current_turn.current_hand().bet; TODO
                    Ok(GameState::PlayerSplit {
                        pending_turns,
                        current_turn,
                        finished_turns,
                        dealer_hand,
                    })
                }
            }
            HandAction::Surrender if self.fast_forward => {
                // Simulated moves should already be valid, so we don't need to check them
                Ok(self.surrender(pending_turns, current_turn, finished_turns, dealer_hand))
            }
            HandAction::Surrender => {
                if let Err(err) = self.check_surrender_allowed(current_turn.current_hand()) {
                    Err((
                        GameState::PlayPlayerTurn {
                            pending_turns,
                            current_turn,
                            finished_turns,
                            dealer_hand,
                        },
                        Error::SurrenderError(err),
                    ))
                } else {
                    Ok(GameState::PlayerSurrender {
                        pending_turns,
                        current_turn,
                        finished_turns,
                        dealer_hand,
                    })
                }
            }
        }
    }

    /// The dealer deals the next card to the player's current hand.
    /// We continue the player's turn if they still have hands in play, or go to the dealer's turn.
    fn hit(
        &mut self,
        pending_turns: Vec<PendingTurn>,
        mut current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        *current_turn.current_hand_mut() += self.shoe.draw_card();
        self.continue_player_phase_or_go_to_dealer(pending_turns, current_turn, finished_turns, dealer_hand)
    }

    /// The player stands and the hand is finished.
    /// We continue the player's turn if they still have hands in play, or go to the dealer's turn.
    fn stand(
        &mut self,
        pending_turns: Vec<PendingTurn>,
        mut current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        current_turn.current_hand_mut().stand();
        self.continue_player_phase_or_go_to_dealer(pending_turns, current_turn, finished_turns, dealer_hand)
    }

    /// The player doubles down and the hand is finished.
    /// We continue the player's turn if they still have hands in play, or go to the dealer's turn.
    fn double(
        &mut self,
        pending_turns: Vec<PendingTurn>,
        mut current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        current_turn.current_hand_mut().double(self.shoe.draw_card());
        self.continue_player_phase_or_go_to_dealer(pending_turns, current_turn, finished_turns, dealer_hand)
    }

    /// The dealer separates the player's hand into two.
    /// Next, the dealer will deal a new card to the first of the two split hands.
    fn split(
        &mut self,
        pending_turns: Vec<PendingTurn>,
        mut current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        let new_hand = current_turn.current_hand_mut().split();
        if self.fast_forward {
            self.deal_first_split_card(pending_turns, current_turn, new_hand, finished_turns, dealer_hand)
        } else {
            GameState::DealFirstSplitCard {
                pending_turns,
                current_turn,
                new_hand,
                finished_turns,
                dealer_hand,
            }
        }
    }

    /// The dealer deals a card to the first of the two split hands.
    /// Next, the dealer will deal a new card to the second of the two split hands.
    fn deal_first_split_card(
        &mut self,
        pending_turns: Vec<PendingTurn>,
        mut current_turn: ActiveTurn,
        new_hand: PlayerHand,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        *current_turn.current_hand_mut() += self.shoe.draw_card();
        if self.fast_forward {
            self.deal_second_split_card(pending_turns, current_turn, new_hand, finished_turns, dealer_hand)
        } else {
            GameState::DealSecondSplitCard {
                pending_turns,
                current_turn,
                new_hand,
                finished_turns,
                dealer_hand,
            }
        }
    }

    /// The dealer deals a card to the second of the two split hands.
    /// The player will play the first split hand first, and the second split hand after.
    /// We continue the player's turn if they still have hands in play, or go to the dealer's turn.
    fn deal_second_split_card(
        &mut self,
        pending_turns: Vec<PendingTurn>,
        mut current_turn: ActiveTurn,
        mut new_hand: PlayerHand,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        new_hand += self.shoe.draw_card();
        current_turn.defer(new_hand);
        self.continue_player_phase_or_go_to_dealer(pending_turns, current_turn, finished_turns, dealer_hand)
    }

    /// The player surrenders and the hand is finished.
    /// We continue the player's turn if they still have hands in play, or go to the dealer's turn.
    fn surrender(
        &mut self,
        pending_turns: Vec<PendingTurn>,
        mut current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        current_turn.current_hand_mut().surrender();
        self.continue_player_phase_or_go_to_dealer(pending_turns, current_turn, finished_turns, dealer_hand)
    }

    /// The dealer reveals their hole card.
    /// If the dealer's hand is no longer in play, the round is over.
    fn play_dealer_turn_or_end_round(
        &mut self,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        if dealer_hand.status == Status::InPlay {
            if self.fast_forward {
                self.play_dealer_turn(finished_turns, dealer_hand)
            } else {
                GameState::PlayDealerTurn {
                    finished_turns,
                    dealer_hand,
                }
            }
        } else {
            if self.fast_forward {
                self.end_round(finished_turns, dealer_hand)
            } else {
                GameState::RoundOver {
                    finished_turns,
                    dealer_hand,
                }
            }
        }
    }

    /// The dealer adds a card to their hand.
    /// This will repeat itself until the dealer's hand is no longer in play.
    /// Then, the round is over.
    fn play_dealer_turn(
        &mut self,
        finished_turns: Vec<FinishedTurn>,
        mut dealer_hand: DealerHand,
    ) -> GameState {
        dealer_hand += self.shoe.draw_card();
        self.play_dealer_turn_or_end_round(finished_turns, dealer_hand)
    }

    /// The round is over, and the dealer turns over their hole card.
    /// The player's total bet and winnings are calculated.
    fn end_round(
        &mut self,
        mut finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    ) -> GameState {
        let total_bets: Vec<u32> = finished_turns.iter().map(|turn| turn.total_bet()).collect();
        let winnings: Vec<u32> = finished_turns.iter()
            .map(|turn| turn.calculate_winnings(&dealer_hand, self.rules.blackjack_payout))
            .collect();
        // let differences: Vec<i32> = total_bets.iter().zip(winnings.iter())
        //     .map(|(bet, win)| *win as i32 - *bet as i32)
        //     .collect();
        self.statistics.update(finished_turns, dealer_hand);
        if self.fast_forward {
            self.pay_out_winnings(winnings)
        } else {
            GameState::Payout {
                total_bets,
                winnings,
            }
        }
    }

    /// The dealer pays out the player's winnings.
    /// If the player has no chips left, the game is over.
    /// Otherwise, the dealer will shuffle the shoe if necessary, or the game will return to betting.
    fn pay_out_winnings(&mut self, _winnings: Vec<u32>) -> GameState {
        // self.chips += total_winnings;
        // if self
        //     .rules
        //     .min_bet
        //     .map_or(self.chips == 0, |min| self.chips < min)
        // {
        //     GameState::GameOver
        // } else 
        if self.shoe.needs_shuffle() {
            if self.fast_forward {
                self.shuffle_dispenser()
            } else {
                GameState::Shuffle
            }
        } else {
            GameState::Betting
        }
    }

    /// The dealer shuffles the shoe.
    /// The game returns to the betting state.
    fn shuffle_dispenser(&mut self) -> GameState {
        self.shoe.shuffle();
        GameState::Betting
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// 
//     #[test]
//     fn test_bet() {
//         let mut table = Table::new(
//             Shoe::new(4, 0.50),
//             Rules {
//                 min_bet: Some(1),
//                 max_bet: Some(100),
//                 ..Rules::default()
//             },
//         );
//         assert_eq!(
//             table.bet(50),
//             Ok(GameState::DealFirstPlayerCard { bet: 50 })
//         );
//         assert_eq!(
//             table.bet(101),
//             Err((GameState::Betting, Error::BetError(BetError::TooHigh)))
//         );
//         assert_eq!(
//             table.bet(0),
//             Err((GameState::Betting, Error::BetError(BetError::TooLow)))
//         );
//         assert_eq!(table.bet(1), Ok(GameState::DealFirstPlayerCard { bet: 1 }));
//         assert_eq!(
//             table.bet(50),
//             Err((GameState::Betting, Error::BetError(BetError::CantAfford)))
//         );
//     }
// }
