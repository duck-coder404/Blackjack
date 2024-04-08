use crate::card::hand::{DealerHand, PlayerHand, PlayerTurn};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum GameState {
    /// We are waiting for the player to place a bet.
    #[default]
    Betting,
    /// We are waiting for the dealer to deal the first card to the player.
    /// The u32 is the player's bet.
    DealFirstPlayerCard(u32),
    /// We are waiting for the dealer to deal the first card to themselves.
    /// The PlayerHand is the player's hand (with the first card).
    DealFirstDealerCard(PlayerHand),
    /// We are waiting for the dealer to deal the second card to the player.
    /// The PlayerHand is the player's one-card hand.
    /// The DealerHand is the dealer's one-card hand.
    DealSecondPlayerCard(PlayerHand, DealerHand),
    /// We are waiting for the dealer to deal the second card to themselves.
    /// The PlayerHand is the player's two-card hand.
    /// The DealerHand is the dealer's one-card hand.
    DealHoleCard(PlayerHand, DealerHand),
    /// The dealer has a 10 or higher showing and has offered the player to surrender early.
    /// We are waiting for the player to decide whether to do so.
    /// The PlayerHand is the player's two-card hand.
    /// The DealerHand is the dealer's two-card hand (hole card is hidden).
    OfferEarlySurrender(PlayerHand, DealerHand),
    /// The dealer has an ace showing and has offered the player insurance.
    /// We are waiting for the player to place an insurance bet (could be 0).
    /// The PlayerHand is the player's two-card hand.
    /// The DealerHand is the dealer's two-card hand (hole card is hidden).
    OfferInsurance(PlayerHand, DealerHand),
    /// We are waiting for the dealer to check their hole card for blackjack.
    /// The PlayerHand is the player's two-card hand.
    /// The DealerHand is the dealer's two-card hand (hole card is hidden).
    /// The u32 is the insurance bet (could be 0).
    CheckDealerHoleCard(PlayerHand, DealerHand, u32),
    /// The dealer does not have blackjack and the player is playing their hand.
    /// We are waiting for the player to make a move.
    /// The PlayerTurn contains the player's original hand plus any splits.
    /// The DealerHand is the dealer's two-card hand (hole card is hidden).
    /// The u32 is the insurance bet (could be 0).
    PlayPlayerTurn(PlayerTurn, DealerHand, u32),
    /// The player has stood.
    /// We wait for dramatic effect.
    Stand(PlayerTurn, DealerHand, u32),
    /// The player has hit.
    /// We are waiting for the dealer to deal the next card to the player's current hand.
    Hit(PlayerTurn, DealerHand, u32),
    /// The player has doubled down.
    /// We are waiting for the dealer to deal the next card to the player's current hand.
    Double(PlayerTurn, DealerHand, u32),
    /// The player has decided to split.
    /// We are waiting for the dealer to separate the player's hand into two.
    Split(PlayerTurn, DealerHand, u32),
    /// The player's hand has been split.
    /// We are waiting for the dealer to deal the new card to their original hand.
    /// The PlayerTurn contains the player's original hand plus any splits.
    /// The PlayerHand is the player's new one-card split hand.
    DealFirstSplitCard(PlayerTurn, PlayerHand, DealerHand, u32),
    /// We are waiting for the dealer to deal the second card to the new split hand.
    /// The PlayerTurn contains the player's original (now two-card) hand plus any splits.
    /// The PlayerHand is the player's new split hand.
    DealSecondSplitCard(PlayerTurn, PlayerHand, DealerHand, u32),
    /// The player has surrendered.
    /// We wait for dramatic effect.
    Surrender(PlayerTurn, DealerHand, u32),
    /// The dealer reveals their hole card.
    /// We are waiting for the dealer to play their hand.
    RevealHoleCard(Vec<PlayerHand>, DealerHand, u32),
    /// The player has finished playing their hands.
    /// We are waiting for the dealer to play.
    /// The Vec<PlayerHand> contains the player's finished hands.
    /// The DealerHand is the dealer's hand with two or more cards.
    PlayDealerTurn(Vec<PlayerHand>, DealerHand, u32),
    /// The dealer has finished playing.
    /// We are waiting for the dealer to pay out winnings.
    /// The Vec<PlayerHand> contains the player's finished hands.
    /// The DealerHand is the dealer's finished hand.
    RoundOver(Vec<PlayerHand>, DealerHand, u32),
    /// The round is over.
    /// We are waiting for the dealer to pay out winnings.
    /// The first u32 is the total bet.
    /// The second u32 is the total winnings.
    Payout(u32, u32),
    /// We are waiting for the dealer to shuffle the shoe.
    Shuffle,
    /// The game is over.
    GameOver,
}
