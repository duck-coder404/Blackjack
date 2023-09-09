use rand::distributions::{Standard, WeightedIndex};
use rand::prelude::*;
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::ops::AddAssign;
use std::str::FromStr;
use std::time::Duration;
use std::{io, thread};

use crate::{Configuration, BlackjackPayout, ShuffleStrategy, Soft17};
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

struct Card {
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

/// Represents the value of a hand, e.g. "Soft 17"
struct Value {
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

struct PlayerHand {
    cards: Vec<Card>,
    value: Value, // (soft, total)
    done: bool,   // stand or bust, cannot hit anymore
    bet: u32,
}

impl PlayerHand {
    fn new(card: Card, bet: u32) -> Self {
        let mut hand = PlayerHand { cards: Vec::new(), value: Value::zero(), done: false, bet };
        hand += card; // Use AddAssign to update the value and announce the card
        hand
    }
    /// Splits the hand
    /// The card taken from this hand is returned, so it can be used to create a new hand
    fn split(&mut self) -> Card {
        assert!(self.is_pair(), "Cannot split hand that is not a pair");
        let split_card = self.cards.pop().unwrap();
        self.value = self.cards[0].value();
        split_card
    }
    /// Compares the hand to the dealer's hand to determine the result
    fn against(&self, dealer: &DealerHand) -> HandResult {
        match (self.status(), dealer.status()) {
            (HandStatus::Blackjack, HandStatus::Blackjack) => HandResult::Push,
            (HandStatus::Blackjack, _) => HandResult::Blackjack,
            (_, HandStatus::Blackjack) => HandResult::Lose,
            (HandStatus::Bust, _) => HandResult::Lose,
            (_, HandStatus::Bust) => HandResult::Win,
            (HandStatus::Ok(player_total), HandStatus::Ok(dealer_total)) => {
                match player_total.cmp(&dealer_total) {
                    Ordering::Less => HandResult::Lose,
                    Ordering::Equal => HandResult::Push,
                    Ordering::Greater => HandResult::Win,
                }
            }
        }
    }
}

impl AddAssign<Card> for PlayerHand {
    fn add_assign(&mut self, rhs: Card) {
        pause();
        self.value += rhs.value();
        let total = self.value.total;
        if total >= 21 {
            self.done = true; // We can't hit anymore
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

impl Hand for PlayerHand {
    fn value(&self) -> &Value {
        &self.value
    }

    fn cards(&self) -> &[Card] {
        &self.cards
    }
}

struct DealerHand {
    cards: Vec<Card>,
    value: Value, // (soft, total)
    done: bool,   // stand or bust, cannot hit anymore
    soft17: Soft17, // Whether the dealer stands or hits on soft 17
}

impl DealerHand {
    fn new(card: Card, soft17: Soft17) -> Self {
        let mut hand = DealerHand { cards: Vec::new(), value: Value::zero(), done: false, soft17 };
        hand += card; // Use AddAssign to update the value and announce the card
        hand
    }
    fn showing(&self) -> u8 {
        self.cards[0].rank.worth()
    }
    fn reveal_down_card(&self) {
        pause();
        print!("The dealer reveals {}. ", self.cards[1]);
        if self.is_21() {
            println!("The dealer has blackjack!");
        } else {
            println!("The dealer has {}.", self.value.total);
        }
    }
}

impl AddAssign<Card> for DealerHand {
    fn add_assign(&mut self, rhs: Card) {
        pause();
        let showing = self.value.total;
        self.value += rhs.value();
        let total = self.value.total;
        self.done = if let (true, 17) = (self.value.soft, total) {
            self.soft17 == Soft17::Stand // Done on soft 17 if dealer stands on soft 17
        } else {
            total >= 17 // Otherwise done if we have any 17 or more
        };
        if self.cards.len() != 1 { // Announce the card unless we are drawing the hidden card
            print!("The dealer draws {}. ", rhs);
            if total > 21 {
                println!("Dealer bust!");
            } else {
                println!("The dealer has {}.", total);
            }
        } else if showing == 10 || showing == 11 {
            println!("The dealer draws a card and checks for blackjack...");
        } else {
            println!("The dealer draws a card.");
        }
        self.cards.push(rhs);
    }
}

impl Hand for DealerHand {
    fn value(&self) -> &Value {
        &self.value
    }
    fn cards(&self) -> &[Card] {
        &self.cards
    }
}

/// The status of a hand is either Ok(total), Blackjack, or Bust
enum HandStatus {
    Ok(u8),
    Blackjack,
    Bust,
}

/// A hand can be compared to another hand to determine the result
/// Blackjack is a special kind of win, as it pays more
enum HandResult {
    Blackjack,
    Win,
    Push,
    Lose,
}

trait Hand {
    fn value(&self) -> &Value;
    fn cards(&self) -> &[Card];
    fn is_blackjack(&self) -> bool {
        self.is_21() && self.len() == 2
    }
    fn is_21(&self) -> bool {
        self.value().total == 21
    }
    fn is_bust(&self) -> bool {
        self.value().total > 21
    }
    fn len(&self) -> usize {
        self.cards().len()
    }
    fn is_pair(&self) -> bool {
        let cards = self.cards();
        cards.len() == 2 && cards[0].rank == cards[1].rank
    }
    /// Returns the status of the hand
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

/// A shoe is a container for cards that can be drawn from
/// It can be shuffled and will reshuffle itself when empty
trait Shoe {
    /// Draws a card from the shoe
    fn draw(&mut self) -> Card;
    /// Shuffles the shoe if it needs shuffling
    fn shuffle_if_needed(&mut self);
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
    shuffle_strategy: ShuffleStrategy,
}

impl MultiDeck {
    /// Create a new multi-deck shoe with the given number of decks and shuffle strategy
    fn new(size: u8, shuffle_strategy: ShuffleStrategy) -> Self {
        let remaining = [size as u16; 52];
        let dist = WeightedIndex::new(remaining).unwrap();
        MultiDeck { size, dist, remaining, shuffle_strategy }
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
    fn shuffle_if_needed(&mut self) {
        match self.shuffle_strategy {
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
        pause();
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
    fn shuffle_if_needed(&mut self) {
        // Do nothing
    }
}

pub fn play(config: Configuration) {
    println!("Welcome to Blackjack!");
    let mut deck: Box<dyn Shoe> = config.decks.into();
    let mut player_chips = config.chips;
    while let Some(bet) = place_bet(player_chips, config.max_bet, config.min_bet) {
        player_chips -= bet;
        println!("You bet {bet} chips. You have {player_chips} chips remaining.");

        let mut player_hand = PlayerHand::new(deck.draw(), bet);
        let mut dealer_hand = DealerHand::new(deck.draw(), config.soft17);

        player_hand += deck.draw();
        dealer_hand += deck.draw();

        // The player may now play their hand, which may turn into multiple hands through splitting
        // (skip if dealer has blackjack)
        let mut player_hands = vec![player_hand];
        if !dealer_hand.is_21() {
            play_hands(&mut player_hands, &dealer_hand, &mut deck, &mut player_chips);
        }

        // At this point, all player hands are done and the dealer reveals their down card
        dealer_hand.reveal_down_card();

        if player_hands.iter().any(|hand| !hand.is_bust() && !hand.is_blackjack()) {
            // At least one hand is not bust and not blackjack, so the dealer must finish their hand
            while !dealer_hand.done {
                dealer_hand += deck.draw();
            }
        }

        // At this point, all hands are done
        // For each hand, determine the result and payout
        let chips_won: u32 = player_hands
            .into_iter()
            .map(|hand| match hand.against(&dealer_hand) {
                HandResult::Blackjack => match config.payout {
                    BlackjackPayout::ThreeToTwo => hand.bet + hand.bet * 3 / 2, // 1.5x win
                    BlackjackPayout::SixToFive => hand.bet + hand.bet * 6 / 5, // 1.2x win
                },
                HandResult::Win => hand.bet + hand.bet, // 1x win
                HandResult::Push => hand.bet, // Bet is returned
                HandResult::Lose => 0, // Bet is lost
            })
            .sum();

        pause();
        match chips_won {
            0 => println!("You lose!"),
            chips if chips < bet => println!("You make back {chips} chips."),
            chips if chips == bet => println!("You push!"),
            chips => println!("You win {chips} chips!"),
        }
        player_chips += chips_won;
        pause();
        deck.shuffle_if_needed();
    }
    println!("You finished with {player_chips} chips.");
    println!("Goodbye!");
    pause();
}

fn play_hands(hands: &mut Vec<PlayerHand>, dealer_hand: &DealerHand, deck: &mut Box<dyn Shoe>, player_chips: &mut u32) {
    while let Some(hand) = hands.iter_mut().find(|hand| !hand.done) {
        pause();
        println!(
            "What would you like to do? ({} against {})",
            hand, dealer_hand.showing()
        );
        match player_action(hand, *player_chips >= hand.bet) {
            Action::Stand => {
                println!("You stand!");
                hand.done = true;
            }
            Action::Hit => {
                println!("You hit!");
                *hand += deck.draw();
            }
            Action::DoubleDown => {
                println!("You double and put another {} chips down!", hand.bet);
                *player_chips -= hand.bet; // The player pays another equal bet
                hand.bet *= 2; // This bet for this hand is now doubled
                *hand += deck.draw();
                hand.done = true;
            }
            Action::Split => {
                println!("You split your hand and put another {} chips down!", hand.bet);
                *player_chips -= hand.bet; // The player pays another equal bet for the new hand
                let mut new_hand = PlayerHand::new(hand.split(), hand.bet);
                *hand += deck.draw();
                new_hand += deck.draw();
                hands.push(new_hand);
            }
        }
    }
}

impl From<Option<u8>> for Box<dyn Shoe> {
    fn from(value: Option<u8>) -> Self {
        match value {
            Some(decks) => Box::new(MultiDeck::new(decks, ShuffleStrategy::Continuous)),
            None => Box::new(InfiniteDeck),
        }
    }
}

/// Prompts the player to place a bet or quit
/// Returns Some(bet) if the player wants to bet bet chips
/// Returns None if the player wants to quit
fn place_bet(chips: u32, max_bet: Option<u32>, min_bet: Option<u32>) -> Option<u32> {
    if chips == 0 {
        println!("You are out of chips!");
        return None;
    }
    println!("You have {chips} chips.");
    println!("How many chips would you like to bet? Type \"stop\" to quit.");
    let mut bet = String::new();
    loop {
        io::stdin()
            .read_line(&mut bet)
            .expect("Failed to read input");
        let trimmed = bet.trim();
        if trimmed == "stop" {
            break None;
        }
        match trimmed.parse() {
            Ok(0) => println!("You must bet at least 1 chip!"),
            Ok(bet) if bet > chips => println!("You don't have enough chips!"),
            Ok(bet) => match (max_bet, min_bet) {
                (Some(max), _) if bet > max => println!("You cannot bet more than {} chips!", max),
                (_, Some(min)) if bet < min => println!("You cannot bet less than {} chips!", min),
                _ => break Some(bet),
            },
            Err(_) => println!("Please enter a number!"),
        }
        bet.clear();
    }
}

/// The actions the player can take on their turn
enum Action {
    Stand,
    Hit,
    DoubleDown,
    Split,
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Action::Stand => write!(f, "Stand (s)"),
            Action::Hit => write!(f, "Hit (h)"),
            Action::DoubleDown => write!(f, "Double Down (d)"),
            Action::Split => write!(f, "Split (p)"),
        }
    }
}

impl FromStr for Action {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "s" => Ok(Action::Stand),
            "stand" => Ok(Action::Stand),
            "h" => Ok(Action::Hit),
            "hit" => Ok(Action::Hit),
            "d" => Ok(Action::DoubleDown),
            "double" => Ok(Action::DoubleDown),
            "p" => Ok(Action::Split),
            "split" => Ok(Action::Split),
            _ => Err(()),
        }
    }
}

/// Prompts the player to choose an action
/// Which actions are available depends on the hand and the player's chips
fn player_action(hand: &PlayerHand, can_double_bet: bool) -> Action {
    let mut action = format!("{}\t\t{}", Action::Hit, Action::Stand);
    if can_double_bet && hand.len() == 2 {
        action.push_str(&format!("\t{}", Action::DoubleDown));
    }
    if can_double_bet && hand.is_pair() {
        action.push_str(&format!("\t\t{}", Action::Split));
    }
    println!("{}", action);
    action.clear(); // Reuse the string
    loop {
        io::stdin()
            .read_line(&mut action)
            .expect("Failed to read input");
        match action.trim().parse() {
            Ok(action) => match action {
                Action::DoubleDown if hand.len() != 2 => {
                    println!("You can only double down on your first two cards!")
                }
                Action::DoubleDown if !can_double_bet => {
                    println!("You don't have enough chips to double down!")
                }
                Action::Split if !hand.is_pair() => println!("You can only split a pair!"),
                Action::Split if !can_double_bet => {
                    println!("You don't have enough chips to split!")
                }
                action => return action,
            },
            Err(_) => println!("Please enter a valid action!"),
        };
        action.clear();
    }
}

fn pause() {
    thread::sleep(Duration::from_secs(1));
}
