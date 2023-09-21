use std::fmt::{self, Display, Formatter};
use std::ops::AddAssign;

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
    fn worth(&self) -> u8 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 10,
            Rank::Queen => 10,
            Rank::King => 10,
            Rank::Ace => 11,
        }
    }

    /// Returns the value of this rank as a hand value.
    /// Only aces are soft.
    fn value(&self) -> Value {
        let soft = self == &Rank::Ace;
        let total = self.worth();
        Value { soft, total }
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
#[derive(Debug, PartialEq)]
pub(crate) struct Card {
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

    /// Returns the value of this card as a hand value.
    /// This is the same as the value of the card's rank.
    fn value(&self) -> Value {
        self.rank.value()
    }
}

impl Display for Card {
    /// Cards are displayed as "a Rank of Suit", e.g. "a Two of Clubs"
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} of {:?}", self.rank, self.suit)
    }
}

/// Represents the game value of a hand
#[derive(Debug, PartialEq, Default)]
pub(crate) struct Value {
    /// Whether the hand has an ace that is currently worth 11
    soft: bool,
    /// The total value of the hand
    total: u8,
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

pub(crate) mod hand {
    use crate::card::{Card, Value};
    use crate::{BlackjackPayout, Soft17};
    use std::cmp::Ordering;
    use std::ops::AddAssign;
    use std::thread;
    use std::time::Duration;

    /// Represents a hand of cards
    pub(crate) trait Hand {
        /// Returns a reference to the value of this hand.
        fn value(&self) -> &Value;

        /// Returns a slice of the cards in this hand.
        fn cards(&self) -> &[Card];

        /// Returns whether this hand has been stood on.
        /// A hand is stood on when the player chooses to stop drawing cards.
        fn is_stood(&self) -> bool;

        /// Returns whether this hand is a blackjack, or "natural" 21.
        /// A blackjack is a hand with only two cards with a total value of 21.
        fn is_blackjack(&self) -> bool {
            self.is_21() && self.cards().len() == 2
        }

        /// Returns whether this hand has a value of 21.
        fn is_21(&self) -> bool {
            self.value().total == 21
        }

        /// Returns whether this hand has bust.
        /// A hand busts when its value exceeds 21.
        fn is_bust(&self) -> bool {
            self.value().total > 21
        }

        /// Returns whether this hand is over, or cannot be played anymore.
        /// This could be because it was stood on, busted, or is a blackjack.
        fn is_over(&self) -> bool {
            self.is_stood() || self.is_bust() || self.is_blackjack()
        }

        /// Returns whether this hand is a pair.
        /// A pair is a hand consisting of only two cards with equal rank.
        fn is_pair(&self) -> bool {
            let cards = self.cards();
            cards.len() == 2 && cards[0].rank == cards[1].rank
        }

        /// Returns the status of this hand.
        /// A hand can be Ok(total), Blackjack, or Bust.
        fn status(&self) -> HandStatus {
            if self.is_blackjack() {
                HandStatus::Blackjack
            } else if self.is_bust() {
                HandStatus::Bust
            } else {
                HandStatus::Ok(self.value().total)
            }
        }
    }

    /// Represents a hand of cards held by the player.
    pub(crate) struct PlayerHand {
        /// The cards in this hand
        cards: Vec<Card>,
        /// The value of this hand
        value: Value,
        /// Whether the player has stood on this hand
        stood: bool,
        /// Whether the player has surrendered this hand
        surrendered: bool,
        /// The player's bet on this hand
        bet: u32,
    }

    impl PlayerHand {
        /// Begins a new hand with the given card and bet.
        pub(crate) fn new(card: Card, bet: u32) -> Self {
            let mut hand = PlayerHand {
                cards: Vec::new(),
                value: Value::default(),
                stood: false,
                surrendered: false,
                bet,
            };
            hand += card; // Use AddAssign to update the hand and announce the card
            hand
        }

        /// Returns the bet on this hand.
        pub(crate) fn bet(&self) -> u32 {
            self.bet
        }

        /// The player stands on this hand.
        pub(crate) fn stand(&mut self) {
            self.stood = true;
        }

        /// The player doubles down on this hand.
        /// The bet is doubled, and the provided card is added to the hand.
        /// If the hand is not bust, the player stands.
        pub(crate) fn double(&mut self, card: Card) {
            self.bet *= 2;
            *self += card;
            self.stood = self.value.total <= 21; // Stand if we didn't bust
        }

        /// The player splits the hand into two hands. This hand must be a pair!
        /// The new hand has the same bet as the original hand.
        pub(crate) fn split(&mut self) -> Self {
            debug_assert!(self.is_pair(), "Cannot split hand that is not a pair");
            let split_card = self.cards.pop().unwrap(); // Remove the second card
            self.value = self.cards[0].value(); // The value of this hand is now the first card
            PlayerHand::new(split_card, self.bet) // Create a new hand with the second card
        }

        /// The player surrenders this hand.
        pub(crate) fn surrender(&mut self) {
            self.surrendered = true;
        }

        /// Returns the winnings of this hand, given the status of the dealer's hand
        /// and the blackjack payout setting.
        pub(crate) fn winnings(&self, dealer_status: &HandStatus, payout: &BlackjackPayout) -> u32 {
            if self.surrendered {
                return self.bet / 2; // Half the bet is returned
            }
            let result = match (self.status(), dealer_status) {
                (HandStatus::Blackjack, HandStatus::Blackjack) => HandResult::Push,
                (HandStatus::Blackjack, _) => HandResult::Blackjack,
                (_, HandStatus::Blackjack) => HandResult::Lose,
                (HandStatus::Bust, _) => HandResult::Lose,
                (_, HandStatus::Bust) => HandResult::Win,
                (HandStatus::Ok(player_total), HandStatus::Ok(dealer_total)) => {
                    match player_total.cmp(dealer_total) {
                        Ordering::Less => HandResult::Lose,
                        Ordering::Equal => HandResult::Push,
                        Ordering::Greater => HandResult::Win,
                    }
                }
            };
            match result {
                HandResult::Blackjack => match payout {
                    BlackjackPayout::ThreeToTwo => self.bet + self.bet * 3 / 2, // 1.5x win
                    BlackjackPayout::SixToFive => self.bet + self.bet * 6 / 5,  // 1.2x win
                },
                HandResult::Win => self.bet + self.bet, // 1x win
                HandResult::Push => self.bet,           // Bet is returned
                HandResult::Lose => 0,                  // Bet is lost
            }
        }
    }

    /// Represents the dealer's hand.
    pub(crate) struct DealerHand {
        /// The cards in this hand
        cards: Vec<Card>,
        /// The value of this hand
        value: Value,
        /// Whether the dealer stands or hits on soft 17
        soft17: Soft17,
    }

    impl DealerHand {
        /// Begins a new hand with the given card and soft 17 setting.
        pub(crate) fn new(card: Card, soft17: Soft17) -> Self {
            let mut hand = DealerHand {
                cards: Vec::new(),
                value: Value::default(),
                soft17,
            };
            hand += card; // Use AddAssign to update the value and announce the card
            hand
        }

        /// Returns the value of the dealer's up card, which is what the player can see.
        pub(crate) fn showing(&self) -> u8 {
            self.cards[0].rank.worth()
        }

        /// Announces the dealer's hole card for the player to see.
        pub(crate) fn reveal_hole_card(&self) {
            thread::sleep(Duration::from_secs(1));
            print!("The dealer reveals {}. ", self.cards[1]);
            if self.is_21() {
                println!("The dealer has blackjack!");
            } else {
                println!("The dealer has {}.", self.value.total);
            }
        }
    }

    impl AddAssign<Card> for PlayerHand {
        /// Adds a card to the player's hand, updating the value and announcing the card.
        fn add_assign(&mut self, rhs: Card) {
            thread::sleep(Duration::from_secs(1));
            self.value += rhs.value();
            print!("You draw {}. ", rhs);
            self.cards.push(rhs);
            match self.value.total {
                22.. => println!("You bust!"),
                21 if self.cards.len() == 2 => println!("Blackjack!"),
                total => println!("You have {}.", total),
            }
        }
    }

    impl AddAssign<Card> for DealerHand {
        /// Adds a card to the dealer's hand, updating the value and announcing the card.
        /// The dealer's second (hole) card is not announced.
        fn add_assign(&mut self, rhs: Card) {
            thread::sleep(Duration::from_secs(1));
            self.value += rhs.value();
            if self.cards.len() != 1 {
                // Announce the card if it is not the dealer's second card
                print!("The dealer draws {}. ", rhs);
                if self.value.total > 21 {
                    println!("Dealer bust!");
                } else {
                    println!("The dealer has {}.", self.value.total);
                }
            } else {
                // The hole card is kept secret until later
                println!("The dealer draws a card.");
            }
            self.cards.push(rhs);
        }
    }

    impl Hand for PlayerHand {
        /// Returns the value of this hand.
        fn value(&self) -> &Value {
            &self.value
        }

        /// Returns the cards in this hand.
        fn cards(&self) -> &[Card] {
            &self.cards
        }

        /// Returns true if the player has stood or surrendered this hand.
        fn is_stood(&self) -> bool {
            self.stood || self.surrendered
        }
    }

    impl Hand for DealerHand {
        /// Returns the value of this hand.
        fn value(&self) -> &Value {
            &self.value
        }

        /// Returns the cards in this hand.
        fn cards(&self) -> &[Card] {
            &self.cards
        }

        /// Returns true if the dealer is required to stand on this hand.
        /// For soft 17s, this depends on the setting.
        /// Otherwise, the dealer stands on any 17 or higher.
        fn is_stood(&self) -> bool {
            match (self.value.soft, self.value.total) {
                (true, 17) => self.soft17 == Soft17::Stand, // Stand on soft 17 if casino rules say so
                (_, total) => total >= 17,              // Otherwise done if we have any 17 or more
            }
        }
    }

    /// Represents the result of comparing a player's hand with the dealer's hand.
    /// A Blackjack is distinct from a Win because it pays more.
    enum HandResult {
        Blackjack,
        Win,
        Push,
        Lose,
    }

    /// Represents the status of a hand, either Ok(total), Blackjack, or Bust.
    pub(crate) enum HandStatus {
        Ok(u8),
        Blackjack,
        Bust,
    }
}

/// A module for dispensing cards.
pub(crate) mod dispenser {
    use crate::card::Card;
    use rand::distributions::WeightedIndex;
    use rand::{thread_rng, Rng};
    use std::thread;
    use std::time::Duration;

    /// Represents a generic dispenser of cards.
    /// Not every dispenser is a shoe, but for now this is the only implementation.
    /// Later, we may want to add specialty dispensers with different distributions.
    pub(crate) struct CardDispenser {
        source: Shoe,
    }

    impl CardDispenser {
        /// Draws a card from the dispenser
        pub(crate) fn draw_card(&mut self) -> Card {
            self.source.draw_card()
        }

        /// Shuffles the dispenser if we have reached the configured shoe penetration
        pub(crate) fn shuffle_if_needed(&mut self, threshold: f32) {
            self.source.shuffle_if_needed(threshold);
        }
    }

    /// Represents a source of cards which can be drawn from.
    trait CardSource {
        /// Draws a card.
        fn draw_card(&mut self) -> Card;

        /// Shuffles the card source if we have used more than the specified proportion of cards.
        fn shuffle_if_needed(&mut self, threshold: f32);

        /// Shuffles the card source. This has the effect of resetting it.
        fn shuffle(&mut self) {}
    }

    /// A shoe is a container that contains multiple decks of cards.
    struct Shoe {
        /// The number of decks in the shoe
        decks: u8,
        /// We use a weighted distribution to draw random cards from the shoe without replacement.
        dist: WeightedIndex<u16>,
        /// The number of each card remaining in the shoe, indexed by ordinal
        remaining: [u16; 52],
    }

    impl Shoe {
        /// Create a new shoe with the given number of decks
        fn with_decks(decks: u8) -> Self {
            let remaining = [decks as u16; 52]; // Start with all cards present
            let dist = WeightedIndex::new(remaining).unwrap();
            Shoe { decks, dist, remaining }
        }
    }

    impl CardSource for Shoe {
        /// Draws a random card from the shoe.
        /// The card is removed from the shoe, and the distribution is updated to reflect the new weight.
        /// If the last card is drawn, the shoe is shuffled.
        fn draw_card(&mut self) -> Card {
            let ordinal = thread_rng().sample(&self.dist);
            self.remaining[ordinal] -= 1; // Remove the card from the shoe
            let new_weight = self.remaining[ordinal];
            // Update the distribution to reflect the new weight of the removed card
            if self.dist.update_weights(&[(ordinal, &new_weight)]).is_err() {
                // The shoe is empty and it is not possible to have empty weights, so shuffle
                println!("The shoe is empty. Shuffling...");
                self.shuffle();
            }
            Card::from_ordinal(ordinal)
        }

        /// Shuffles the shoe if we have reached the configured shoe penetration.
        fn shuffle_if_needed(&mut self, threshold: f32) {
            let shoe_size = self.decks as u16 * 52;
            let cards_played = shoe_size - self.remaining.iter().sum::<u16>();
            let penetration = cards_played as f32 / shoe_size as f32;
            if penetration >= threshold {
                println!("The shoe is shuffled...");
                self.shuffle();
            }
        }

        /// Shuffles the shoe.
        fn shuffle(&mut self) {
            thread::sleep(Duration::from_secs(1));
            self.remaining = [self.decks as u16; 52];
            self.dist = WeightedIndex::new(self.remaining).unwrap();
        }
    }

    impl From<u8> for CardDispenser {
        /// Creates a new card dispenser with the given number of decks.
        fn from(decks: u8) -> Self {
            CardDispenser { source: Shoe::with_decks(decks) }
        }
    }
}
