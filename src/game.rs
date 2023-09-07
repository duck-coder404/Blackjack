use std::fmt::{self, Display, Formatter};
use std::ops::{Add, AddAssign};
use std::str::FromStr;
use std::time::Duration;
use std::{io, thread};
use std::cmp::Ordering;
use rand::prelude::*;
use rand::distributions::{Standard, WeightedIndex};

use rand::Rng;
use crate::{Blackjack, BlackjackPayout, ShuffleStrategy, Soft17};

#[derive(Debug, PartialEq, Clone, Copy)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Distribution<Suit> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Suit {
        Suit::from_ordinal(rng.gen_range(0..=3))
    }
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

#[derive(Debug, PartialEq, Clone, Copy)]
enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,  // 10
    Queen, // 10
    King,  // 10
    Ace,   // 11/1
}

impl Distribution<Rank> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Rank {
        Rank::from_ordinal(rng.gen_range(0..=12))
    }
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
    fn value(&self) -> u8 {
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

impl PartialOrd for Rank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.value().cmp(&other.value()))
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

impl Add for Rank {
    type Output = (bool, u8); // (soft, total)

    fn add(self, rhs: Self) -> Self::Output {
        let soft = self == Rank::Ace || rhs == Rank::Ace;
        let mut total = self.value() + rhs.value();
        if total == 22 {
            // Two aces
            total -= 10;
        }
        (soft, total)
    }
}

#[derive(Debug, PartialEq)]
struct Card {
    rank: Rank,
    suit: Suit,
}

impl Distribution<Card> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Card {
        Card::from_ordinal(rng.gen_range(0..=51))
    }
}

impl Card {
    fn from_ordinal(ordinal: usize) -> Self {
        let rank = Rank::from_ordinal(ordinal / 4);
        let suit = Suit::from_ordinal(ordinal % 4);
        Card { rank, suit }
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_ordinal = self.rank as usize * 4 + self.suit as usize;
        let other_ordinal = other.rank as usize * 4 + other.suit as usize;
        Some(self_ordinal.cmp(&other_ordinal))
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} of {:?}", self.rank, self.suit)
    }
}

struct DealerCard(Card);
struct PlayerCard(Card);

impl Add<DealerCard> for DealerCard {
    type Output = Hand;

    fn add(self, rhs: DealerCard) -> Self::Output {
        // It takes the dealer a second to add the cards together
        thread::sleep(Duration::from_secs(1));
        let DealerCard(lhs) = self;
        let DealerCard(rhs) = rhs;
        let (soft, total) = lhs.rank + rhs.rank;
        // Do not announce the total for the dealer's first hand, the second card is still hidden
        let cards = vec![lhs, rhs];
        Hand {
            cards,
            soft,
            total,
            done: total >= 17,
        } // Dealer stands on 17
    }
}

/// The player's first two cards are added together to form a hand
impl Add<PlayerCard> for PlayerCard {
    type Output = Hand;

    fn add(self, rhs: PlayerCard) -> Self::Output {
        thread::sleep(Duration::from_secs(1));
        let PlayerCard(lhs) = self;
        let PlayerCard(rhs) = rhs;
        let (soft, total) = lhs.rank + rhs.rank;
        if total == 21 {
            println!("Blackjack!");
        } else {
            println!("You have {}.", total);
        }
        let cards = vec![lhs, rhs];
        Hand {
            cards,
            soft,
            total,
            done: total == 21,
        }
    }
}

macro_rules! add_assign_impl {
    ($t:ty, $normal:expr, $bust:expr) => {
        impl AddAssign<$t> for Hand {
            fn add_assign(&mut self, rhs: $t) {
                if rhs.0.rank == Rank::Ace { // We drew an ace
                    if self.total <= 10 {
                        self.total += 11; // We had 10 or less, the new ace is worth 11
                        self.soft = true; // And we now have a soft hand
                    } else {
                        self.total += 1; // We had 11 or more, the new ace is worth 1
                    }
                } else {
                    self.total += rhs.0.rank.value();
                }
                if self.total > 21 && self.soft {
                    // We would have busted, but there is a soft ace to save us
                    self.total -= 10;
                    self.soft = false;
                }
                if self.total >= 21 {
                    self.done = true; // We can't hit anymore
                }
                self.cards.push(rhs.0);
                if self.total > 21 {
                    println!($bust, self.total);
                } else {
                    println!($normal, self.total);
                }
            }
        }
    };
}

add_assign_impl!(PlayerCard, "You have {}.", "{}! You bust!");
add_assign_impl!(DealerCard, "The dealer has {}.", "{}! Dealer bust!");

struct Hand {
    cards: Vec<Card>,
    soft: bool,
    total: u8,
    done: bool, // stand or bust, cannot hit anymore
}

enum HandStatus {
    Ok(u8),
    Blackjack,
    Bust,
}

enum HandResult {
    Blackjack,
    Win,
    Push,
    Lose,
}

impl Hand {
    fn is_21(&self) -> bool {
        self.total == 21
    }
    fn is_bust(&self) -> bool {
        self.total > 21
    }
    fn is_two_cards(&self) -> bool {
        self.cards.len() == 2
    }
    fn is_pair(&self) -> bool {
        self.is_two_cards() && self.cards[0].rank == self.cards[1].rank
    }
    fn split_and_replace(&mut self, card: PlayerCard) -> PlayerCard {
        assert!(self.is_pair(), "Cannot split hand that is not a pair");
        let right = self.cards.pop().unwrap();
        self.total -= right.rank.value();
        if self.soft {
            // Splitting two aces
            self.total += 10;
        }
        *self += card;
        PlayerCard(right)
    }
    fn status(&self) -> HandStatus {
        if self.is_21() && self.is_two_cards() {
            HandStatus::Blackjack
        } else if self.is_bust() {
            HandStatus::Bust
        } else {
            HandStatus::Ok(self.total)
        }
    }
    fn against(&self, dealer: &Hand) -> HandResult {
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

impl Display for Hand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            if self.soft { "Soft" } else { "Hard" },
            self.total
        )
    }
}

trait Shoe {
    fn draw(&mut self) -> Card;
    fn shuffle_if_needed(&mut self);
    fn shuffle(&mut self) {}

    fn draw_player(&mut self) -> PlayerCard {
        thread::sleep(Duration::from_secs(1));
        let card = self.draw();
        println!("You draw {card}.");
        PlayerCard(card)
    }
    fn draw_dealer(&mut self, hidden: bool) -> DealerCard {
        thread::sleep(Duration::from_secs(1));
        let card = self.draw();
        if hidden {
            println!("The dealer draws a hidden card.");
        } else {
            println!("The dealer draws {card}.");
        }
        DealerCard(card)
    }
}

struct MultiDeck {
    size: u8,
    dist: WeightedIndex<u16>,
    remaining: [u16; 52], // Amount of each card remaining, indexed by ordinal
    shuffle_strategy: ShuffleStrategy,
}

impl MultiDeck {
    fn new(size: u8, shuffle_strategy: ShuffleStrategy) -> Self {
        let remaining = [size as u16; 52];
        let dist = WeightedIndex::new(remaining).unwrap();
        MultiDeck { size, dist, remaining, shuffle_strategy }
    }
}

impl Shoe for MultiDeck {
    fn draw(&mut self) -> Card {
        let ordinal = self.dist.sample(&mut thread_rng());
        self.remaining[ordinal] -= 1;
        let new_weight = self.remaining[ordinal];
        if self.dist.update_weights(&[(ordinal, &new_weight)]).is_err() {
            println!("The shoe is empty. Shuffling...");
            self.shuffle();
        }
        Card::from_ordinal(ordinal)
    }
    fn shuffle_if_needed(&mut self) {
        match self.shuffle_strategy {
            ShuffleStrategy::Continuous => self.shuffle(),
            ShuffleStrategy::QuarterShoe => if count(&self.remaining) <= self.size as u16 * 39 {
                println!("The shoe is a quarter empty. Shuffling...");
                self.shuffle();
            },
            ShuffleStrategy::HalfShoe => if count(&self.remaining) <= self.size as u16 * 26 {
                println!("The shoe is half empty. Shuffling...");
                self.shuffle();
            },
            ShuffleStrategy::ThreeQuartersShoe => if count(&self.remaining) <= self.size as u16 * 13 {
                println!("The shoe is three quarters empty. Shuffling...");
                self.shuffle();
            },
            ShuffleStrategy::EmptyShoe => if count(&self.remaining) <= 1 {
                println!("The shoe is empty. Shuffling...");
                self.shuffle();
            },
        }
        fn count(remaining: &[u16; 52]) -> u16 {
            remaining.iter().sum()
        }
    }
    fn shuffle(&mut self) {
        thread::sleep(Duration::from_secs(2));
        self.remaining = [self.size as u16; 52];
        self.dist = WeightedIndex::new(self.remaining).unwrap();
    }
}

struct InfiniteDeck;

impl Shoe for InfiniteDeck {
    fn draw(&mut self) -> Card {
        random()
    }
    fn shuffle_if_needed(&mut self) {
        // Do nothing
    }
}

pub fn play(config: Blackjack) {
    println!("Welcome to Blackjack!");
    let mut deck: Box<dyn Shoe> = match config.decks {
        Some(decks) => Box::new(MultiDeck::new(decks, config.shuffle)),
        None => Box::new(InfiniteDeck),
    };
    let mut player_chips = config.chips;
    while let Some(mut bet) = place_bet(player_chips, config.max_bet, config.min_bet) {
        player_chips -= bet;
        println!("You bet {bet} chips. You have {player_chips} chips remaining.");

        let player_card = deck.draw_player();
        let dealer_card = deck.draw_dealer(false);

        let player_hand = player_card + deck.draw_player();
        let mut dealer_hand = dealer_card + deck.draw_dealer(true);

        let dealer_showing = dealer_hand.cards[0].rank.value();
        // In reality, the dealer would check for blackjack here
        if dealer_showing == 10 || dealer_showing == 11 {
            thread::sleep(Duration::from_secs(1));
            println!("The dealer checks their hidden card...");
        }

        // The player may now play their hand(s) (skip if dealer has blackjack)
        let mut hands = vec![player_hand];
        if !dealer_hand.is_21() {
            while let Some(hand) = hands.iter_mut().find(|hand| !hand.done) {
                println!(
                    "What would you like to do? ({} against {})",
                    hand,
                    dealer_hand.cards[0].rank.value()
                );
                match player_action(hand, bet <= player_chips) {
                    Action::Stand => {
                        println!("You stand!");
                        hand.done = true;
                    }
                    Action::Hit => {
                        println!("You hit!");
                        *hand += deck.draw_player();
                    }
                    Action::DoubleDown => {
                        println!("You double and put another {bet} chips down!");
                        player_chips -= bet;
                        // FIXME: If this is a split hand the doubled bet will apply to both the player's hands
                        bet *= 2; // Double the bet for this hand
                        *hand += deck.draw_player();
                        hand.done = true;
                    }
                    Action::Split => {
                        println!("You split your hand into two and put another {bet} chips down!");
                        player_chips -= bet; // Do not double the bet, each hand is worth the original bet
                        let right = hand.split_and_replace(deck.draw_player());
                        let right_hand = right + deck.draw_player();
                        hands.push(right_hand);
                    }
                }
            }
        }

        // At this point, all player hands are done and the dealer reveals their second card
        thread::sleep(Duration::from_secs(1));
        println!("The dealer reveals {}.", dealer_hand.cards[1]);
        if dealer_hand.is_21() {
            println!("The dealer has blackjack!");
        } else {
            println!("The dealer has {}.", dealer_hand.total);
        }

        if hands.iter().any(|hand| !hand.is_bust()) {
            // At least one hand is not bust, so the dealer must play
            while !dealer_hand.done { // Keep drawing cards until the dealer is done
                dealer_hand += deck.draw_dealer(false);

                match (dealer_hand.soft, dealer_hand.total) {
                    (_, total) if total < 17 => {}
                    (true, 17) if config.soft17 == Soft17::Hit => {} // Soft 17 hit if config says so
                    _ => dealer_hand.done = true, // Dealer stands
                }
            }
        }

        // At this point, all hands are done
        // For each hand, determine the result and payout
        let chips_won: u32 = hands
            .into_iter()
            .map(|hand| hand.against(&dealer_hand))
            .map(|result| match result {
                HandResult::Blackjack => match config.payout {
                    BlackjackPayout::ThreeToTwo => bet + bet * 3 / 2,
                    BlackjackPayout::SixToFive => bet + bet * 6 / 5,
                },
                HandResult::Win => bet + bet,
                HandResult::Push => bet,
                HandResult::Lose => 0,
            })
            .sum();

        match chips_won {
            0 => println!("You lose!"),
            chips if chips < bet => println!("You make back {chips} chips."),
            chips if chips == bet => println!("You push!"),
            chips => println!("You win {chips} chips!"),
        }
        player_chips += chips_won;
        deck.shuffle_if_needed();
    }
    println!("You finished with {player_chips} chips.");
    println!("Goodbye!");
    thread::sleep(Duration::from_secs(1));
}

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
            Ok(bet) => {
                match (max_bet, min_bet) {
                    (Some(max), _) if bet > max => println!("You cannot bet more than {} chips!", max),
                    (_, Some(min)) if bet < min => println!("You cannot bet less than {} chips!", min),
                    _ => break Some(bet),
                }
            },
            Err(_) => println!("Please enter a number!"),
        }
        bet.clear();
    }
}

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

fn player_action(hand: &Hand, can_double_bet: bool) -> Action {
    print!("{}\t\t{}", Action::Hit, Action::Stand);
    if can_double_bet && hand.is_two_cards() {
        print!("\t{}", Action::DoubleDown);
    }
    if can_double_bet && hand.is_pair() {
        print!("\t\t{}", Action::Split);
    }
    println!();
    let mut action = String::new();
    loop {
        io::stdin()
            .read_line(&mut action)
            .expect("Failed to read input");
        match action.trim().parse() {
            Ok(action) => match action {
                Action::DoubleDown => {
                    if !hand.is_two_cards() {
                        println!("You can only double down on your first two cards!");
                    } else if !can_double_bet {
                        println!("You don't have enough chips to double down!");
                    } else {
                        break action;
                    }
                }
                Action::Split => {
                    if !hand.is_pair() {
                        println!("You can only split a pair!");
                    } else if !can_double_bet {
                        println!("You don't have enough chips to split!");
                    } else {
                        break action;
                    }
                }
                action => break action,
            },
            Err(_) => {
                println!("Please enter a valid action!");
            }
        };
        action.clear();
    }
}
