use crate::card::hand::{DealerHand, PlayerHand, PlayerTurn};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum GameState {
    /// We wait for the player to place a bet.
    #[default]
    Betting,
    /// We wait for the dealer to deal the first card to the player.
    DealFirstPlayerCard { bet: u32 },
    /// We wait for the dealer to deal the first card to themselves.
    DealFirstDealerCard { player_hand: PlayerHand },
    /// We wait for the dealer to deal the second card to the player.
    DealSecondPlayerCard {
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
    },
    /// We wait for the dealer to deal the second card to themselves.
    DealHoleCard {
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
    },
    /// The dealer has a 10 or higher showing and has offered the player to surrender early.
    /// We wait for the player to decide whether to do so.
    OfferEarlySurrender {
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
    },
    /// The dealer has an ace showing and has offered the player insurance.
    /// We wait for the player to place an insurance bet (could be 0).
    OfferInsurance {
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
    },
    /// We wait for the dealer to check their hole card for blackjack.
    CheckDealerHoleCard {
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The dealer does not have blackjack and the player is playing their hand.
    /// We wait for the player to make a move.
    PlayPlayerTurn {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player has stood on their current hand.
    /// We wait for dramatic effect.
    Stand {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player has hit on their current hand.
    /// We wait for the dealer to deal the next card to the player's current hand.
    Hit {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player has doubled down on their current hand.
    /// We wait for the dealer to deal the next card to the player's current hand.
    Double {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player has decided to split their current hand.
    /// We wait for the dealer to split the hand into two separate hands of one card each.
    Split {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player's current hand has been split, and a new hand has been created.
    /// Next, the dealer will deal a card to the hand which was split (not the new hand yet).
    DealFirstSplitCard {
        player_turn: PlayerTurn,
        new_hand: PlayerHand,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// We wait for the dealer to deal the second card to the new split hand.
    /// Next, the new hand will become part of the player's hands.
    DealSecondSplitCard {
        player_turn: PlayerTurn,
        new_hand: PlayerHand,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player has surrendered their current hand.
    /// We wait for dramatic effect.
    Surrender {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The dealer reveals their hole card.
    /// We wait for the dealer to play their hand.
    RevealHoleCard {
        finished_hands: Vec<PlayerHand>,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player has finished playing their hands.
    /// We wait for the dealer to play.
    PlayDealerTurn {
        finished_hands: Vec<PlayerHand>,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The dealer has finished playing.
    /// We wait for the dealer to pay out winnings.
    RoundOver {
        finished_hands: Vec<PlayerHand>,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The round is over.
    /// We wait for the dealer to pay out winnings.
    /// The first u32 is the total bet.
    /// The second u32 is the total winnings.
    Payout { total_bet: u32, total_winnings: u32 },
    /// We wait for the dealer to shuffle the shoe.
    Shuffle,
    /// The game is over.
    GameOver,
}
