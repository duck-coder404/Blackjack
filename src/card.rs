use std::fmt::{self, Display, Formatter};
use crate::card::hand::Value;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Suit {
    Clubs, Diamonds, Hearts, Spades
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Rank {
    Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace
}

impl Rank {
    /// Returns how much a card with this rank is worth in the game.
    /// All face cards are worth 10, and aces are worth 11.
    fn worth(self) -> u8 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
            Rank::Ace => 11,
        }
    }
}

impl Display for Rank {
    /// Ranks are displayed as "a Rank", e.g. "a Two", "a Seven", "an Eight", "an Ace"
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Rank::Two => write!(f, "a Two"),
            Rank::Three => write!(f, "a Three"),
            Rank::Four => write!(f, "a Four"),
            Rank::Five => write!(f, "a Five"),
            Rank::Six => write!(f, "a Six"),
            Rank::Seven => write!(f, "a Seven"),
            Rank::Eight => write!(f, "an Eight"),
            Rank::Nine => write!(f, "a Nine"),
            Rank::Ten => write!(f, "a Ten"),
            Rank::Jack => write!(f, "a Jack"),
            Rank::Queen => write!(f, "a Queen"),
            Rank::King => write!(f, "a King"),
            Rank::Ace => write!(f, "an Ace"),
        }
    }
}

/// A card is a combination of a rank and a suit.
/// Do not implement Copy or Clone for this type, as it is important that cards are not duplicated.
#[derive(Debug, PartialEq)]
pub struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    /// Returns the card corresponding to the given ordinal value (0-51).
    /// The ordinal value is the index of the card in a deck sorted by rank and then suit,
    /// e.g. twos first, then threes, fours, etc.
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
        Card { rank, suit }
    }

    /// Returns the value of this card as if it were alone in a hand.
    /// Aces are soft 11s, while all other cards are hard values.
    pub fn value(&self) -> Value {
        let soft = self.rank == Rank::Ace;
        let total = self.rank.worth();
        Value { soft, total }
    }
}

impl Display for Card {
    /// Cards are displayed as "a Rank of Suit", e.g. "a Two of Clubs"
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} of {:?}", self.rank, self.suit)
    }
}

pub mod hand {
    use crate::card::Card;
    use std::cmp::Ordering;
    use std::fmt;
    use std::fmt::{Display, Formatter};
    use std::ops::{Add, AddAssign};

    /// Represents the game value of a hand, e.g. "Soft 20"
    #[derive(Debug, PartialEq, Default)]
    pub struct Value {
        /// Whether the hand has an ace that is currently worth 11
        pub soft: bool,
        /// The total value of the hand
        pub total: u8,
    }

    impl AddAssign for Value {
        /// Adds two hand values together, taking care to handle soft values and avoid busting if possible
        fn add_assign(&mut self, mut rhs: Self) {
            // Check if adding the totals would bust
            if rhs.soft && self.total + rhs.total > 21 {
                // If rhs is soft, we can subtract 10 to avoid busting
                rhs.total -= 10;
                rhs.soft = false;
            }
            if self.soft && self.total + rhs.total > 21 {
                // If that didn't work, we can try again if self is also soft
                self.total -= 10;
                self.soft = false;
            }
            self.total += rhs.total;
            self.soft |= rhs.soft;
        }
    }

    impl Display for Value {
        /// A hand is displayed as "Soft/Hard total", e.g. "Soft 20"
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{} {}", if self.soft { "Soft" } else { "Hard" }, self.total)
        }
    }

    /// Represents the state of a hand.
    /// A hand in play must be acted upon until it is in a terminal state.
    #[derive(PartialEq, Default)]
    pub enum Status {
        #[default]
        InPlay,
        Stood,
        Blackjack,
        Bust,
        Surrendered,
    }

    /// Represents the dealer's hand.
    #[derive(Default)]
    pub struct DealerHand {
        /// The cards in this hand
        pub cards: Vec<Card>,
        /// The value of this hand
        pub value: Value,
        /// The status of this hand
        pub status: Status,
        /// Whether the dealer stands or hits on soft 17
        soft_17_hit: bool,
    }

    impl DealerHand {
        /// Creates a new dealer hand with the given card and hit-on-soft-17 setting.
        pub fn new(card: Card, soft_17_hit: bool) -> Self {
            DealerHand {
                soft_17_hit,
                ..Default::default()
            } + card // Add the card to the hand to initialize it
        }

        /// Returns the worth of the dealer's up card, which is what the player must base their decisions on.
        pub fn showing(&self) -> u8 {
            self.cards[0].rank.worth()
        }

        /// Announces the dealer's hole card and total.
        pub fn reveal_hole_card(&self) {
            print!("The dealer reveals {}. ", self.cards[1]);
            if self.status == Status::Blackjack {
                println!("The dealer has blackjack!");
            } else {
                println!("The dealer has {}.", self.value.total);
            }
        }
    }

    impl AddAssign<Card> for DealerHand {
        /// Adds a card to the dealer's hand, updating the value and announcing the card.
        /// If this is the dealer's second card, it is not announced.
        fn add_assign(&mut self, rhs: Card) {
            self.value += rhs.value();
            if self.cards.len() == 1 {
                // The hole card is kept secret until later
                println!("The dealer draws a card.");
            } else {
                // Announce the card if it is not the dealer's second card
                print!("The dealer draws {rhs}. ");
                match self.value.total {
                    22.. => println!("The dealer busts!"),
                    total => println!("The dealer has {total}."),
                }
            }
            self.cards.push(rhs);
            self.status = match (self.value.soft, self.value.total) {
                (_, 22..) => Status::Bust,
                (true, 21) if self.cards.len() == 2 => Status::Blackjack,
                (true, 17) if self.soft_17_hit => Status::InPlay,
                (_, 17..) => Status::Stood,
                _ => Status::InPlay,
            };
        }
    }

    impl Add<Card> for DealerHand {
        type Output = DealerHand;

        /// Adds a card to the dealer's hand, updating the value and announcing the card.
        /// The dealer's second (hole) card is not announced.
        fn add(mut self, rhs: Card) -> Self::Output {
            self += rhs;
            self
        }
    }

    /// Represents a hand of cards held by the player.
    #[derive(Default)]
    pub struct PlayerHand {
        /// The cards in this hand
        pub cards: Vec<Card>,
        /// The value of this hand
        pub value: Value,
        /// The status of this hand
        pub status: Status,
        /// Whether this hand was split from another hand (important for doubling down)
        pub splits: u8,
        /// The player's bet on this hand
        pub bet: u32,
        /// The player's winnings on this hand
        pub winnings: u32,
    }

    impl PlayerHand {
        /// Creates a new player hand with the given card and bet.
        pub fn new(card: Card, bet: u32) -> Self {
            PlayerHand {
                bet,
                ..Default::default()
            } + card // Add the card to the hand to initialize it
        }

        /// The player stands on this hand.
        pub fn stand(&mut self) {
            self.status = Status::Stood;
        }

        /// The player doubles down on this hand.
        /// The bet is doubled, and the provided card is added to the hand.
        /// If the hand is not bust, the player stands.
        pub fn double(&mut self, card: Card) {
            self.bet *= 2;
            *self += card;
            if let Status::InPlay = self.status {
                self.status = Status::Stood;
            }
        }

        /// The player splits the hand into two hands. This hand must be a pair!
        /// The new hand has the same bet as the original hand.
        pub fn split(&mut self) -> Self {
            debug_assert!(self.is_pair(), "Cannot split hand that is not a pair");
            let split_card = self.cards.pop().unwrap(); // Remove the second card
            self.value = self.cards[0].value(); // The value of this hand is now the first card
            // Create a new hand with the second card
            PlayerHand {
                splits: self.splits + 1, // Increment the number of splits
                bet: self.bet,
                ..Default::default()
            } + split_card // Add the split card to the new hand to update it
        }

        /// The player surrenders this hand.
        pub fn surrender(&mut self) {
            self.status = Status::Surrendered;
        }

        /// Returns whether this hand is a pair.
        /// A pair is a hand consisting of only two cards with equal rank.
        pub fn is_pair(&self) -> bool {
            self.cards.len() == 2 && self.cards[0].rank == self.cards[1].rank
        }

        /// Returns whether this hand is composed of the given two card worths.
        /// This method is only meant to be used in the context of a match statement
        /// where the hand is known to have two cards with the total worth1 + worth2.
        pub fn composed_of(&self, worth1: u8, worth2: u8) -> bool {
            let first_worth = self.cards[0].rank.worth();
            first_worth == worth1 || first_worth == worth2
        }

        /// Calculates the winnings for this hand based on the dealer's hand.
        /// This method should only be called once the dealer's hand is in a terminal state.
        pub fn calculate_winnings(&mut self, dealer_hand: &DealerHand, six_to_five: bool) {
            self.winnings = match (&self.status, &dealer_hand.status) {
                (Status::Surrendered, _) => self.surrender_payout(), // Surrender
                (Status::Blackjack, Status::Blackjack) => self.bet, // Blackjack push
                (Status::Blackjack, _) => self.blackjack_payout(six_to_five), // Blackjack win
                (_, Status::Blackjack) | (Status::Bust, _) => 0, // Dealer blackjack or player bust
                (_, Status::Bust) => self.win_payout(), // Dealer bust
                _ => {
                    match self.value.total.cmp(&dealer_hand.value.total) {
                        Ordering::Greater => self.win_payout(), // Player win
                        Ordering::Equal => self.bet, // Push
                        Ordering::Less => 0, // Dealer win
                    }
                }
            }
        }

        /// Calculates the winnings for a blackjack win based on whether the game pays 3:2 or 6:5.
        fn blackjack_payout(&self, six_to_five: bool) -> u32 {
            if six_to_five { self.bet + self.bet * 6 / 5 } else { self.bet + self.bet * 3 / 2 }
        }

        /// Calculates the winnings for a normal win.
        fn win_payout(&self) -> u32 {
            self.bet * 2
        }

        /// Calculates the winnings for a surrender.
        fn surrender_payout(&self) -> u32 {
            self.bet / 2
        }
    }

    impl AddAssign<Card> for PlayerHand {
        /// Adds a card to the player's hand, updating the value and announcing the card.
        fn add_assign(&mut self, rhs: Card) {
            print!("You draw {rhs}. ");
            self.value += rhs.value();
            self.cards.push(rhs);
            self.status = match self.value.total {
                22.. => {
                    println!("You bust!");
                    Status::Bust
                },
                21 if self.cards.len() == 2 => {
                    println!("Blackjack!");
                    Status::Blackjack
                },
                21 => {
                    println!("You have 21.");
                    Status::Stood
                },
                total => {
                    println!("You have {total}.");
                    Status::InPlay
                },
            }
        }
    }

    impl Add<Card> for PlayerHand {
        type Output = PlayerHand;

        /// Adds a card to the player's hand, updating the value and announcing the card.
        fn add(mut self, rhs: Card) -> Self::Output {
            self += rhs;
            self
        }
    }
}

/// A module for dispensing cards.
pub mod dispenser {
    use crate::card::Card;
    use rand::distributions::WeightedIndex;
    use rand::{thread_rng, Rng};

    /// A shoe is a container that contains multiple decks of cards.
    pub struct Shoe {
        /// The number of decks in the shoe
        pub decks: u8,
        /// Weighted distribution to draw random cards from the shoe without replacement.
        dist: WeightedIndex<u16>,
        /// The number of each card remaining in the shoe, indexed by ordinal
        /// This is initialized to the number of decks in the shoe
        remaining: [u16; 52],
        /// The proportion of cards to play before shuffling
        shuffle_threshold: f32,
    }

    impl Shoe {
        /// Create a new shoe with the given number of decks
        pub fn new(decks: u8, shuffle_threshold: f32) -> Self {
            let remaining = [u16::from(decks); 52]; // Start with all cards present
            let dist = WeightedIndex::new(remaining).unwrap();
            Shoe { decks, dist, remaining, shuffle_threshold }
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
                println!("The shoe is empty. Shuffling...");
                self.shuffle();
            }
            Card::from_ordinal(ordinal)
        }

        /// Checks if the shoe needs to be shuffled.
        pub fn needs_shuffle(&mut self) -> bool {
            let shoe_size = u16::from(self.decks) * 52;
            let cards_played = shoe_size - self.remaining.iter().sum::<u16>();
            let penetration = f32::from(cards_played) / f32::from(shoe_size);
            penetration >= self.shuffle_threshold
        }

        /// Shuffles the shoe.
        pub fn shuffle(&mut self) {
            self.remaining = [u16::from(self.decks); 52];
            self.dist = WeightedIndex::new(self.remaining).unwrap();
        }
    }
}
