use crate::card::hand::{DealerHand, PlayerHand, PlayerTurn};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum GameState {
    /// The player is placing a bet.
    #[default]
    Betting,
    /// The dealer is dealing the first card to the player.
    DealFirstPlayerCard { bet: u32 },
    /// The dealer is dealing the first card to themselves.
    DealFirstDealerCard { player_hand: PlayerHand },
    /// The dealer is dealing the second card to the player.
    DealSecondPlayerCard {
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
    },
    /// The dealer deals the hole card to themselves.
    DealHoleCard {
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
    },
    /// The player has a chance to surrender early (before the dealer checks for blackjack).
    OfferEarlySurrender {
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
    },
    /// The dealer offers the player insurance because they have an ace showing.
    OfferInsurance {
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
    },
    /// The dealer checks their hole card to see if they have blackjack.
    CheckDealerHoleCard {
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player is choosing their action for their current hand.
    PlayPlayerTurn {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player chooses to stand on their current hand.
    PlayerStand {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player chooses to hit on their current hand.
    PlayerHit {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player chooses to double down on their current hand.
    PlayerDouble {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player chooses to split their current hand.
    PlayerSplit {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The dealer is dealing the first card to the newly split hand.
    DealFirstSplitCard {
        player_turn: PlayerTurn,
        new_hand: PlayerHand,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The dealer has dealt the first card to the split hand, and is now dealing the second card.
    DealSecondSplitCard {
        player_turn: PlayerTurn,
        new_hand: PlayerHand,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The player chooses to surrender (late) on their current hand.
    PlayerSurrender {
        player_turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The dealer reveals their hole card.
    RevealHoleCard {
        finished_hands: Vec<PlayerHand>,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The dealer draws a card.
    PlayDealerTurn {
        finished_hands: Vec<PlayerHand>,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The dealer has finished playing and the round is over.
    RoundOver {
        finished_hands: Vec<PlayerHand>,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    },
    /// The dealer is paying out the winnings.
    Payout { total_bet: u32, total_winnings: u32 },
    /// The dealer is shuffling the shoe.
    Shuffle,
    /// The game is over.
    GameOver,
}
