use std::fmt::{self, Display, Formatter};
use std::ops::AddAssign;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Suit {
    Clubs, Diamonds, Hearts, Spades,
}

impl Suit {
    fn from_ordinal(ordinal: usize) -> Self {
        match ordinal {
            0 => Suit::Clubs,
            1 => Suit::Diamonds,
            2 => Suit::Hearts,
            3 => Suit::Spades,
            _ => panic!("Invalid ordinal"),
        }
    }
}

/// Allows us to draw a randomly generated suit
impl Distribution<Suit> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Suit {
        Suit::from_ordinal(rng.gen_range(0..=3))
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Rank {
    Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace,
}

impl Rank {
    fn from_ordinal(ordinal: usize) -> Self {
        match ordinal {
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
        }
    }
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
}

/// Allows us to draw a randomly generated rank
impl Distribution<Rank> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Rank {
        Rank::from_ordinal(rng.gen_range(0..=12))
    }
}

impl Display for Rank {
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

pub(crate) struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    fn from_ordinal(ordinal: usize) -> Self {
        let rank = Rank::from_ordinal(ordinal / 4);
        let suit = Suit::from_ordinal(ordinal % 4);
        Card { rank, suit }
    }
    fn value(&self) -> Value {
        let soft = self.rank == Rank::Ace;
        let total = self.rank.worth();
        Value { soft, total }
    }
}

/// Allows us to draw a randomly generated card
impl Distribution<Card> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Card {
        Card::from_ordinal(rng.gen_range(0..=51))
    }
}

/// Cards are displayed as "a Rank of Suit", e.g. "a Two of Clubs"
impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} of {:?}", self.rank, self.suit)
    }
}

/// Represents the game value of a card or multiple cards
pub(crate) struct Value {
    soft: bool,
    total: u8,
}

impl Value {
    fn zero() -> Self {
        Value { soft: false, total: 0 }
    }
}

impl AddAssign for Value {
    fn add_assign(&mut self, mut rhs: Self) {
        self.total += rhs.total;
        // If rhs is soft, we can subtract 10 to avoid busting
        if self.total > 21 && rhs.soft {
            self.total -= 10;
            rhs.soft = false;
        }
        // If that didn't work, we can try again if self is also soft
        if self.total > 21 && self.soft {
            self.total -= 10;
            self.soft = false;
        }
        self.soft = self.soft || rhs.soft; // If one remains soft, the result is soft
    }
}

pub(crate) mod hand {
    use std::cmp::Ordering;
    use std::fmt::{self, Display, Formatter};
    use std::ops::AddAssign;
    use std::thread;
    use std::time::Duration;
    use crate::card::{Card, Value};
    use crate::{BlackjackPayout, Soft17};

    pub(crate) trait Hand {
        fn value(&self) -> &Value;
        fn cards(&self) -> &[Card];
        fn is_stood(&self) -> bool;
        fn is_blackjack(&self) -> bool {
            self.is_21() && self.len() == 2
        }
        fn is_21(&self) -> bool {
            self.value().total == 21
        }
        fn is_bust(&self) -> bool {
            self.value().total > 21
        }
        fn is_over(&self) -> bool {
            self.is_stood() || self.is_bust() || self.is_blackjack()
        }
        fn len(&self) -> usize {
            self.cards().len()
        }
        fn is_pair(&self) -> bool {
            let cards = self.cards();
            cards.len() == 2 && cards[0].rank == cards[1].rank
        }
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

    pub(crate) struct PlayerHand {
        cards: Vec<Card>,
        value: Value, // (soft, total)
        stood: bool,   // Whether the player has stood on this hand
        surrendered: bool,
        bet: u32,
    }

    impl PlayerHand {
        pub(crate) fn new(card: Card, bet: u32) -> Self {
            let mut hand = PlayerHand {
                cards: Vec::new(),
                value: Value::zero(),
                stood: false,
                surrendered: false,
                bet
            };
            hand += card; // Use AddAssign to update the hand and announce the card
            hand
        }
        pub(crate) fn bet(&self) -> u32 {
            self.bet
        }
        pub(crate) fn stand(&mut self) {
            self.stood = true;
        }
        pub(crate) fn double(&mut self, card: Card) {
            self.bet += self.bet;
            *self += card;
            self.stood = self.value.total <= 21; // Stand if we didn't bust
        }
        /// Splits the hand
        /// The card taken from this hand is returned, so it can be used to create a new hand
        pub(crate) fn split(hand: &mut PlayerHand) -> PlayerHand {
            debug_assert!(hand.is_pair(), "Cannot split hand that is not a pair");
            let split_card = hand.cards.pop().unwrap();
            hand.value = hand.cards[0].value();
            PlayerHand::new(split_card, hand.bet)
        }
        pub(crate) fn surrender(&mut self) {
            self.surrendered = true;
        }
        /// Compares the status of this hand to the status of the dealer's hand
        /// and computes the winnings
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
                    BlackjackPayout::SixToFive => self.bet + self.bet * 6 / 5, // 1.2x win
                },
                HandResult::Win => self.bet + self.bet, // 1x win
                HandResult::Push => self.bet, // Bet is returned
                HandResult::Lose => 0, // Bet is lost
            }
        }
    }

    pub(crate) struct DealerHand {
        cards: Vec<Card>,
        value: Value, // (soft, total)
        soft17: Soft17, // Whether the dealer stands or hits on soft 17
    }

    impl DealerHand {
        pub(crate) fn new(card: Card, soft17: Soft17) -> Self {
            let mut hand = DealerHand { cards: Vec::new(), value: Value::zero(), soft17 };
            hand += card; // Use AddAssign to update the value and announce the card
            hand
        }
        pub(crate) fn showing(&self) -> u8 {
            self.cards[0].rank.worth()
        }
        pub(crate) fn reveal_down_card(&self) {
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
        fn add_assign(&mut self, rhs: Card) {
            thread::sleep(Duration::from_secs(1));
            self.value += rhs.value();
            let total = self.value.total;
            if total >= 21 {
                self.stood = true; // We can't hit anymore
            }
            print!("You draw {}. ", rhs);
            self.cards.push(rhs);
            match total {
                22.. => println!("You bust!"),
                21 if self.len() == 2 => println!("Blackjack!"),
                _ => println!("You have {}.", total),
            }
        }
    }

    impl AddAssign<Card> for DealerHand {
        fn add_assign(&mut self, rhs: Card) {
            thread::sleep(Duration::from_secs(1));
            self.value += rhs.value();
            let total = self.value.total;
            if self.cards.len() != 1 { // Announce the card unless it is the second (down) card
                print!("The dealer draws {}. ", rhs);
                if total > 21 {
                    println!("Dealer bust!");
                } else {
                    println!("The dealer has {}.", total);
                }
            } else {
                println!("The dealer draws a card.");
            }
            self.cards.push(rhs);
        }
    }

    impl Hand for PlayerHand {
        fn value(&self) -> &Value {
            &self.value
        }
        fn cards(&self) -> &[Card] {
            &self.cards
        }
        fn is_stood(&self) -> bool {
            self.stood || self.surrendered
        }
    }

    impl Hand for DealerHand {
        fn value(&self) -> &Value {
            &self.value
        }
        fn cards(&self) -> &[Card] {
            &self.cards
        }
        fn is_stood(&self) -> bool {
            if let (true, 17) = (self.value.soft, self.value.total) {
                self.soft17 == Soft17::Stand // Done on soft 17 if dealer stands on soft 17
            } else {
                self.value.total >= 17 // Otherwise done if we have any 17 or more
            }
        }
    }

    /// A hand can be compared to another hand to determine the result
    /// Blackjack is a special kind of win, as it pays more
    enum HandResult {
        Blackjack,
        Win,
        Push,
        Lose,
    }

    /// The status of a hand is either Ok(total), Blackjack, or Bust
    pub(crate) enum HandStatus {
        Ok(u8),
        Blackjack,
        Bust,
    }

    impl Display for PlayerHand {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{} {}",
                if self.value.soft { "Soft" } else { "Hard" },
                self.value.total
            )
        }
    }
}

pub(crate) mod shoe {
    use std::thread;
    use std::time::Duration;
    use rand::distributions::WeightedIndex;
    use rand::{random, Rng, thread_rng};
    use crate::card::Card;
    use crate::ShuffleStrategy;

    /// A shoe is a container for cards that can be drawn from
    /// It can be shuffled and will reshuffle itself when empty
    pub(crate) trait Shoe {
        /// Draws a card from the shoe
        fn draw(&mut self) -> Card;
        /// Shuffles the shoe if it needs shuffling
        fn shuffle_if_needed(&mut self, shuffle_strategy: &ShuffleStrategy);
        /// Shuffles the shoe
        fn shuffle(&mut self) {}
    }

    /// A multi-deck shoe is a shoe that contains multiple decks of cards
    /// As it is drawn from, the cards are removed from the shoe
    /// Shuffling replaces all the cards in the shoe
    struct MultiDeck {
        size: u8,
        dist: WeightedIndex<u16>,
        remaining: [u16; 52], // Amount of each card remaining, indexed by ordinal
    }

    impl MultiDeck {
        /// Create a new multi-deck shoe with the given number of decks and shuffle strategy
        fn with_size(size: u8) -> Self {
            let remaining = [size as u16; 52];
            let dist = WeightedIndex::new(remaining).unwrap();
            MultiDeck { size, dist, remaining }
        }
    }

    impl Shoe for MultiDeck {
        fn draw(&mut self) -> Card {
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
        fn shuffle_if_needed(&mut self, shuffle_strategy: &ShuffleStrategy) {
            match shuffle_strategy {
                ShuffleStrategy::Continuous => self.shuffle(),
                ShuffleStrategy::QuarterShoe => {
                    if count(&self.remaining) <= self.size as u16 * (52 * 3 / 4) {
                        println!("The shoe is a quarter empty. Shuffling...");
                        self.shuffle();
                    }
                }
                ShuffleStrategy::HalfShoe => {
                    if count(&self.remaining) <= self.size as u16 * (52 / 2) {
                        println!("The shoe is half empty. Shuffling...");
                        self.shuffle();
                    }
                }
                ShuffleStrategy::ThreeQuartersShoe => {
                    if count(&self.remaining) <= self.size as u16 * (52 / 4) {
                        println!("The shoe is three quarters empty. Shuffling...");
                        self.shuffle();
                    }
                }
                ShuffleStrategy::EmptyShoe => {
                    if count(&self.remaining) <= 1 {
                        println!("The shoe is empty. Shuffling...");
                        self.shuffle();
                    }
                }
            }
            fn count(remaining: &[u16; 52]) -> u16 {
                remaining.iter().sum()
            }
        }
        fn shuffle(&mut self) {
            thread::sleep(Duration::from_secs(1));
            self.remaining = [self.size as u16; 52];
            self.dist = WeightedIndex::new(self.remaining).unwrap();
        }
    }

    /// An infinite deck never runs out of cards and is always uniformly distributed
    struct InfiniteDeck;

    impl Shoe for InfiniteDeck {
        /// Draws a random card from the infinite deck
        fn draw(&mut self) -> Card {
            random()
        }
        /// An infinite deck never needs shuffling
        fn shuffle_if_needed(&mut self, _shuffle_strategy: &ShuffleStrategy) {
            // Do nothing
        }
    }

    impl From<Option<u8>> for Box<dyn Shoe> {
        fn from(value: Option<u8>) -> Self {
            match value {
                Some(decks) => Box::new(MultiDeck::with_size(decks)),
                None => Box::new(InfiniteDeck),
            }
        }
    }
}
