use crate::card::hand::{DealerHand, PlayerHand, ActiveTurn, PendingTurn, FinishedTurn};

/// The state of a round of Blackjack.
/// This does not including the betting phase, which is before the round starts.
#[derive(Debug, PartialEq, Eq)]
pub enum GameState {
    /// The round has not yet started. The players are placing their bets.
    Betting,
    /// The dealer is dealing the first cards to the players.
    /// Each player has placed a bet and will receive a hand one by one.
    /// The bets vector starts full and is emptied as the players are dealt their hands.
    /// The player_hands vector starts empty and is filled as the players are dealt their hands.
    DealFirstPlayerCards {
        bets: Vec<u32>, // The players who have not yet received a card
        player_turns: Vec<PendingTurn> // The players who have received a card
    },
    /// The dealer is dealing the first card to themselves.
    DealFirstDealerCard {
        player_turns: Vec<PendingTurn>, // Every player has 1 card
    },
    /// The dealer is dealing the second card to the player.
    DealSecondPlayerCards {
        player_turns: Vec<PendingTurn>, // Every player has 1 card, will receive a second
        dealer_hand: DealerHand, // Dealer has 1 card
    },
    /// The dealer deals the hole card to themselves.
    DealHoleCard {
        player_turns: Vec<PendingTurn>, // Every player has 2 cards
        dealer_hand: DealerHand, // Dealer has 1 card, will receive a second
    },
    /// The dealer is showing an ace or ten.
    /// The players have a chance to surrender before the dealer checks for blackjack.
    /// The PlayerHands will move from not_offered_yet to already_offered as they are offered surrender.
    OfferEarlySurrender {
        player_turns: Vec<PendingTurn>,
        dealer_hand: DealerHand,
    },
    /// The dealer has an ace showing and offers the players insurance.
    /// The players put in their insurance bets simultaneously.
    OfferInsurance {
        player_turns: Vec<PendingTurn>,
        dealer_hand: DealerHand,
    },
    /// The dealer checks their hole card to see if they have blackjack.
    CheckDealerHoleCard {
        player_turns: Vec<PendingTurn>, // Hand and insurance bet for each player
        dealer_hand: DealerHand,
    },
    /// The players are playing their hands.
    /// `player_turn` is the turn of the player whose move it is.
    /// `already_played` is the list of turns which are no longer in play.
    /// `not_played_yet` is the list of hands which have not yet been played.
    PlayPlayerTurn {
        pending_turns: Vec<PendingTurn>,
        current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The player chooses to stand on their current hand.
    PlayerStand {
        pending_turns: Vec<PendingTurn>,
        current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The player chooses to hit on their current hand.
    PlayerHit {
        pending_turns: Vec<PendingTurn>,
        current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The player chooses to double down on their current hand.
    PlayerDouble {
        pending_turns: Vec<PendingTurn>,
        current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The player chooses to split their current hand.
    PlayerSplit {
        pending_turns: Vec<PendingTurn>,
        current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The dealer is dealing the first card to the newly split hand.
    DealFirstSplitCard {
        pending_turns: Vec<PendingTurn>,
        current_turn: ActiveTurn,
        new_hand: PlayerHand,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The dealer has dealt the first card to the split hand, and is now dealing the second card.
    DealSecondSplitCard {
        pending_turns: Vec<PendingTurn>,
        current_turn: ActiveTurn,
        new_hand: PlayerHand,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The player chooses to surrender (late) on their current hand.
    PlayerSurrender {
        pending_turns: Vec<PendingTurn>,
        current_turn: ActiveTurn,
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The dealer reveals their hole card.
    RevealHoleCard {
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The dealer draws a card.
    PlayDealerTurn {
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The dealer has finished playing and the round is over.
    RoundOver {
        finished_turns: Vec<FinishedTurn>,
        dealer_hand: DealerHand,
    },
    /// The dealer is paying out the winnings.
    /// The indices of the total_bets and winnings vectors correspond to the players.
    Payout {
        total_bets: Vec<u32>,
        winnings: Vec<u32>
    },
    /// The dealer is shuffling the shoe.
    Shuffle,
}
