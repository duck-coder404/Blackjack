use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Suit {
    Clubs, Diamonds, Hearts, Spades
}

impl fmt::Display for Suit {
    /// Suits are displayed as their name, e.g. "Clubs", "Diamonds", "Hearts", "Spades"
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clubs => write!(f, "Clubs"),
            Self::Diamonds => write!(f, "Diamonds"),
            Self::Hearts => write!(f, "Hearts"),
            Self::Spades => write!(f, "Spades"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rank {
    Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace
}

impl fmt::Display for Rank {
    /// Ranks are displayed as "a Rank", e.g. "a Two", "a Seven", "an Eight", "an Ace"
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Two => write!(f, "a Two"),
            Self::Three => write!(f, "a Three"),
            Self::Four => write!(f, "a Four"),
            Self::Five => write!(f, "a Five"),
            Self::Six => write!(f, "a Six"),
            Self::Seven => write!(f, "a Seven"),
            Self::Eight => write!(f, "an Eight"),
            Self::Nine => write!(f, "a Nine"),
            Self::Ten => write!(f, "a Ten"),
            Self::Jack => write!(f, "a Jack"),
            Self::Queen => write!(f, "a Queen"),
            Self::King => write!(f, "a King"),
            Self::Ace => write!(f, "an Ace"),
        }
    }
}

impl Rank {
    /// Returns how much a card with this rank is worth in the game.
    /// All face cards are worth 10, and aces are worth 11.
    #[must_use]
    pub const fn worth(&self) -> u8 {
        match self {
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
            Self::Ten | Self::Jack | Self::Queen | Self::King => 10,
            Self::Ace => 11,
        }
    }
}

/// A card is a combination of a rank and a suit.
/// Do not implement Copy for this type, as it is important that cards are not duplicated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl fmt::Display for Card {
    /// Cards are displayed as "a Rank of Suit", e.g. "a Two of Clubs"
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} of {}", self.rank, self.suit)
    }
}

impl Card {
    /// Returns the card corresponding to the given ordinal value (0-51).
    /// The ordinal value is the index of the card in a deck sorted by rank and then suit,
    /// e.g. twos first, then threes, fours, etc.
    ///
    /// # Panics
    ///
    /// Panics if `ordinal` is >= 52
    fn from_ordinal(ordinal: usize) -> Self {
        let rank = match ordinal / 4 {
            0 => Rank::Two,
            1 => Rank::Three,
            2 => Rank::Four,
            3 => Rank::Five,
            4 => Rank::Six,
            5 => Rank::Seven,
            6 => Rank::Eight,
            7 => Rank::Nine,
            8 => Rank::Ten,
            9 => Rank::Jack,
            10 => Rank::Queen,
            11 => Rank::King,
            12 => Rank::Ace,
            _ => panic!("Invalid ordinal"),
        };
        let suit = match ordinal % 4 {
            0 => Suit::Clubs,
            1 => Suit::Diamonds,
            2 => Suit::Hearts,
            3 => Suit::Spades,
            _ => panic!("Invalid ordinal"),
        };
        Self { rank, suit }
    }
}

pub mod hand {
    use std::cmp::Ordering;
    use std::fmt;
    use std::ops::{Add, AddAssign};

    use crate::card::{Card, Rank};
    use crate::rules::{BlackjackPayout, DealerSoft17Action};

    /// Represents the game value of a hand, e.g. "Soft 20"
    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct Value {
        /// Whether the hand has an ace that is currently worth 11
        pub soft: bool,
        /// The total value of the hand
        pub total: u8,
    }

    impl Value {
        /// Creates a new Value from the given card.
        /// Aces are soft.
        fn new(card: &Card) -> Self {
            Self {
                soft: card.rank == Rank::Ace,
                total: card.rank.worth(),
            }
        }
    }

    impl AddAssign<&Card> for Value {
        /// Adds two hand values together, taking care to handle soft values and avoid busting if possible
        fn add_assign(&mut self, rhs: &Card) {
            let Self { mut soft, total: mut card_value } = Self::new(rhs);
            // Prevent busting by converting the soft ace to a hard ace
            if soft && self.total + card_value > 21 {
                card_value -= 10; // Convert the ace from 11 to 1
                soft = false;
            }
            // Prevent busting by converting the current hand's soft ace to a hard ace
            if self.soft && self.total + card_value > 21 {
                self.total -= 10; // Convert the ace from 11 to 1
                self.soft = false;
            }
            self.total += card_value; // Add the card's worth to the total
            self.soft |= soft; // If either hand has a soft ace, the result is a soft hand
        }
    }

    impl fmt::Display for Value {
        /// A hand is displayed as "Soft/Hard total", e.g. "Soft 20"
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{} {}",
                if self.soft { "Soft" } else { "Hard" },
                self.total
            )
        }
    }

    /// Represents the status of a hand.
    /// A hand may still be in play, or it may be in any of the four terminal states.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub enum Status {
        #[default]
        InPlay,
        Stood,
        Bust,
        Blackjack,
        Surrendered,
    }

    /// Represents the dealer's hand.
    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct DealerHand {
        /// The cards in this hand
        pub cards: Vec<Card>,
        /// The value of this hand
        pub value: Value,
        /// The status of this hand
        pub status: Status,
        /// Whether the dealer stands or hits on soft 17
        soft_17_action: DealerSoft17Action,
    }

    impl AddAssign<Card> for DealerHand {
        /// Adds a card to the dealer's hand, updating the value and announcing the card.
        /// If this is the dealer's second card, it is not announced.
        fn add_assign(&mut self, rhs: Card) {
            debug_assert_eq!(self.status, Status::InPlay, "cannot add to finished hand");
            self.value += &rhs;
            self.cards.push(rhs);
            self.status = match (self.value.soft, self.value.total) {
                (true, 17) if self.hits_on_soft_17() => Status::InPlay,
                (true, 21) if self.size() == 2 => Status::Blackjack,
                (_, 17..=21) => Status::Stood,
                (_, 22..) => Status::Bust,
                _ => Status::InPlay,
            };
        }
    }

    impl Add<Card> for DealerHand {
        type Output = Self;

        /// Adds a card to the dealer's hand, updating the value and announcing the card.
        /// The dealer's second (hole) card is not announced.
        fn add(mut self, rhs: Card) -> Self::Output {
            self += rhs;
            self
        }
    }

    impl DealerHand {
        /// Creates a new dealer hand with the given card and hit-on-soft-17 setting.
        #[must_use]
        pub fn new(card: Card, soft_17_action: DealerSoft17Action) -> Self {
            Self {
                soft_17_action,
                ..Self::default()
            } + card // Add the card to the hand to initialize it
        }

        /// Returns the number of cards in this hand.
        #[must_use]
        pub fn size(&self) -> usize {
            self.cards.len()
        }

        /// Returns the worth of the dealer's up card, which is what the player must base their decisions on.
        #[must_use]
        pub fn showing(&self) -> u8 {
            self.cards[0].rank.worth()
        }
        
        /// Returns whether the dealer hits on soft 17.
        #[must_use]
        pub fn hits_on_soft_17(&self) -> bool {
            self.soft_17_action == DealerSoft17Action::Hit
        }
    }

    /// Represents a hand of cards held by the player.
    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct PlayerHand {
        /// The cards in this hand
        pub cards: Vec<Card>,
        /// The value of this hand
        pub value: Value,
        /// The status of this hand
        pub status: Status,
        /// The number of times this hand has been split
        pub splits: u8,
        /// The player's bet on this hand
        pub bet: u32,
        /// The player's winnings on this hand
        pub winnings: u32,
    }

    impl AddAssign<Card> for PlayerHand {
        /// Adds a card to the player's hand, updating the value and announcing the card.
        fn add_assign(&mut self, rhs: Card) {
            debug_assert_eq!(self.status, Status::InPlay, "cannot add to finished hand");
            self.value += &rhs;
            self.cards.push(rhs);
            self.status = match self.value.total {
                22.. => Status::Bust,
                21 if self.size() == 2 => Status::Blackjack,
                21 => Status::Stood,
                _ => Status::InPlay,
            }
        }
    }

    impl Add<Card> for PlayerHand {
        type Output = Self;

        /// Adds a card to the player's hand, updating the value and announcing the card.
        fn add(mut self, rhs: Card) -> Self::Output {
            self += rhs;
            self
        }
    }

    impl PlayerHand {
        /// Creates a new player hand with the given card and bet.
        #[must_use]
        pub fn new(card: Card, bet: u32) -> Self {
            Self {
                bet,
                ..Self::default()
            } + card // Add the card to the hand to initialize it
        }

        /// The player stands on this hand.
        pub fn stand(&mut self) {
            debug_assert_eq!(self.status, Status::InPlay, "cannot stand on finished hand");
            self.status = Status::Stood;
        }

        /// The player doubles down on this hand.
        /// The bet is doubled, and the provided card is added to the hand.
        /// If the hand is not bust, the player stands.
        pub fn double(&mut self, card: Card) {
            debug_assert_eq!(self.size(), 2, "cannot double down on hand with more than two cards");
            debug_assert_eq!(self.status, Status::InPlay, "cannot double down on finished hand");
            self.bet *= 2;
            *self += card;
            if self.status == Status::InPlay {
                self.status = Status::Stood;
            }
        }

        /// The player splits the hand into two hands. This hand must be a pair!
        /// The new hand has the same bet as the original hand.
        /// # Panics
        /// Will panic if the hand is not a pair.
        #[must_use]
        pub fn split(&mut self) -> Self {
            debug_assert!(self.is_pair(), "cannot split hand that is not a pair");
            let split_card = self.cards.pop().expect("Hand must be a pair"); // Remove the second card
            self.value = Value::new(&self.cards[0]); // The value of this hand is now the first card
            // Create a new hand with the second card
            Self {
                splits: self.splits + 1, // Increment the number of splits
                bet: self.bet,
                ..Self::default()
            } + split_card // Add the split card to the new hand to update it
        }

        /// The player surrenders this hand.
        pub fn surrender(&mut self) {
            debug_assert_eq!(self.size(), 2, "cannot surrender on hand with more than two cards");
            self.status = Status::Surrendered;
        }

        /// Returns the number of cards in this hand.
        #[must_use]
        pub fn size(&self) -> usize {
            self.cards.len()
        }

        /// Returns whether this hand is a pair.
        /// A pair is a hand consisting of only two cards with equal rank.
        #[must_use]
        pub fn is_pair(&self) -> bool {
            self.size() == 2 && self.cards[0].rank == self.cards[1].rank
        }

        /// Calculates the winnings for this hand based on the dealer's hand.
        /// This method should only be called once the dealer's hand is in a terminal state.
        #[must_use]
        pub fn calculate_winnings(
            &self,
            dealer_hand: &DealerHand,
            blackjack_payout: BlackjackPayout,
        ) -> u32 {
            match (&self.status, &dealer_hand.status) {
                (Status::Surrendered, _) => self.payout_surrender(), // Surrender
                (Status::Blackjack, Status::Blackjack) => self.payout_push(), // Blackjack push
                (Status::Blackjack, _) => self.payout_blackjack(blackjack_payout), // Blackjack win
                (_, Status::Blackjack) | (Status::Bust, _) => self.payout_loss(), // Dealer blackjack or player bust
                (_, Status::Bust) => self.payout_win(),                           // Dealer bust
                _ => match self.value.total.cmp(&dealer_hand.value.total) {
                    Ordering::Greater => self.payout_win(), // Player win
                    Ordering::Equal => self.payout_push(),  // Push
                    Ordering::Less => self.payout_loss(),   // Dealer win
                },
            }
        }

        /// Calculates the winnings for a blackjack win based on whether the game pays 3:2 or 6:5.
        const fn payout_blackjack(&self, payout: BlackjackPayout) -> u32 {
            match payout {
                BlackjackPayout::ThreeToTwo => self.bet + self.bet * 3 / 2,
                BlackjackPayout::SixToFive => self.bet + self.bet * 6 / 5,
            }
        }

        /// Calculates the winnings for a normal win, which is double the bet.
        const fn payout_win(&self) -> u32 {
            self.bet * 2
        }

        /// Calculates the winnings for a push, which is the same as the bet.
        const fn payout_push(&self) -> u32 {
            self.bet
        }

        /// Calculates the winnings for a surrender, which is half the bet.
        const fn payout_surrender(&self) -> u32 {
            self.bet / 2
        }

        /// Calculates the winnings for a loss, which is 0.
        const fn payout_loss(&self) -> u32 {
            0
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct PlayerHands {
        pending: Vec<PlayerHand>,
        pub current: PlayerHand,
        finished: Vec<PlayerHand>,
    }

    impl PlayerHands {
        #[must_use]
        pub fn new(current: PlayerHand) -> Self {
            Self {
                pending: Vec::new(),
                current,
                finished: Vec::new(),
            }
        }
        
        /// Defer the provided hand to be played after the current hand.
        pub fn defer(&mut self, hand: PlayerHand) {
            self.pending.push(hand);
        }

        /// Continues playing the current hand if it is in play.
        /// If the current hand is finished, it is moved to the finished hands
        /// and the next pending hand becomes the current hand.
        /// If there are no more pending hands, the finished hands are returned.
        /// # Errors
        /// If the current hand is finished and there are no more pending hands,
        /// the finished hands are returned as an error.
        pub fn continue_playing(mut self) -> Result<Self, Vec<PlayerHand>> {
            if self.current.status == Status::InPlay {
                Ok(self)
            } else {
                self.finished.push(self.current);
                while let Some(hand) = self.pending.pop() {
                    if hand.status == Status::InPlay {
                        self.current = hand;
                        return Ok(self);
                    }
                    self.finished.push(hand);
                }
                Err(self.finished)
            }
        }
    }
}

pub mod shoe {
    use rand_distr::WeightedTreeIndex;
    use rand::{thread_rng, Rng};

    use crate::card::Card;

    /// A shoe is a container that contains multiple decks of cards.
    #[derive(Debug, Clone)]
    pub struct Shoe {
        /// The number of decks in the shoe
        pub decks: u8,
        /// Weighted distribution to draw random cards from the shoe without replacement.
        dist: WeightedTreeIndex<u16>,
        /// The number of each card remaining in the shoe, indexed by ordinal
        /// This is initialized to the number of decks in the shoe
        remaining: [u16; 52],
        /// The proportion of cards to play before shuffling
        shuffle_threshold: f32,
    }

    impl Shoe {
        /// Create a new shoe with the given number of decks and shuffle threshold.
        /// The shoe is initialized with all cards present.
        /// # Panics
        ///
        /// Panics if the number of decks is 0
        #[must_use]
        pub fn new(decks: u8, shuffle_threshold: f32) -> Self {
            let remaining = [u16::from(decks); 52]; // Start with all cards present
            let dist = WeightedTreeIndex::new(remaining).unwrap();
            Self {
                decks,
                dist,
                remaining,
                shuffle_threshold,
            }
        }

        /// Draws a random card from the shoe.
        /// The card is removed from the shoe, and the distribution is updated to reflect the new weight.
        /// If the last card is drawn, the shoe is shuffled.
        pub fn draw_card(&mut self) -> Card {
            let ordinal = thread_rng().sample(&self.dist);
            self.remaining[ordinal] -= 1; // Remove the card from the shoe
            let new_weight = self.remaining[ordinal];
            // Update the distribution to reflect the new weight of the removed card
            if self.dist.update_weights(&[(ordinal, &new_weight)]).is_err() {
                // The update failed, so we must have drawn the last card
                self.shuffle();
            }
            Card::from_ordinal(ordinal)
        }

        /// Checks if the shoe needs to be shuffled.
        #[must_use]
        pub fn needs_shuffle(&self) -> bool {
            let shoe_size = u16::from(self.decks) * 52;
            let cards_played = shoe_size - self.remaining.iter().sum::<u16>();
            let penetration = f32::from(cards_played) / f32::from(shoe_size);
            penetration >= self.shuffle_threshold
        }

        /// Shuffles the shoe.
        /// All cards are returned to the shoe, and the distribution is updated to reflect the new weights.
        /// # Panics
        ///
        /// Panics if the number of decks is 0
        pub fn shuffle(&mut self) {
            self.remaining = [u16::from(self.decks); 52];
            self.dist = WeightedTreeIndex::new(self.remaining).unwrap();
        }
    }
}
