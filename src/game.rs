use std::fmt::{self, Display, Formatter};
use std::ops::{Add, AddAssign};
use std::str::FromStr;
use std::time::Duration;
use std::{io, thread};
use std::cmp::Ordering;
use rand::prelude::*;
use rand::distributions::WeightedIndex;

use rand::Rng;
use crate::{Blackjack, ShuffleStrategy, StandOn};

#[derive(Debug, PartialEq)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    #[allow(dead_code)]
    fn random() -> Self {
        let ordinal = thread_rng().gen_range(0..=3);
        Suit::from_ordinal(ordinal)
    }
    fn ordinal(&self) -> usize {
        match self {
            Suit::Clubs => 0,
            Suit::Diamonds => 1,
            Suit::Hearts => 2,
            Suit::Spades => 3,
        }
    }
    fn from_ordinal(ordinal: usize) -> Self {
        match ordinal {
            0 => Suit::Clubs,
            1 => Suit::Diamonds,
            2 => Suit::Hearts,
            3 => Suit::Spades,
            _ => unreachable!(),
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

impl Rank {
    #[allow(dead_code)]
    fn random() -> Self {
        let ordinal = thread_rng().gen_range(0..=12);
        Rank::from_ordinal(ordinal)
    }
    fn ordinal(&self) -> usize {
        match self {
            Rank::Two => 0,
            Rank::Three => 1,
            Rank::Four => 2,
            Rank::Five => 3,
            Rank::Six => 4,
            Rank::Seven => 5,
            Rank::Eight => 6,
            Rank::Nine => 7,
            Rank::Ten => 8,
            Rank::Jack => 9,
            Rank::Queen => 10,
            Rank::King => 11,
            Rank::Ace => 12,
        }
    }
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

impl Card {
    fn random() -> Self {
        let ordinal = thread_rng().gen_range(0..=51);
        Card::from_ordinal(ordinal)
    }
    fn ordinal(&self) -> usize {
        self.rank.ordinal() * 4 + self.suit.ordinal()
    }
    fn from_ordinal(ordinal: usize) -> Self {
        let rank = Rank::from_ordinal(ordinal / 4);
        let suit = Suit::from_ordinal(ordinal % 4);
        Card { rank, suit }
    }
    #[allow(dead_code)]
    fn with_rank(rank: Rank) -> Self {
        let suit = Suit::random();
        Card { rank, suit }
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.ordinal().cmp(&other.ordinal()))
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

impl Hand {
    fn is_blackjack(&self) -> bool {
        self.is_21() && self.is_two_cards()
    }
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
    fn winnings_against(&self, dealer_hand: &Hand, bet: u32) -> u32 {
        // Dealer is not blackjack
        if self.is_blackjack() {
            bet * 5 / 2
        } else if self.is_bust() {
            0
        } else if dealer_hand.is_bust() {
            bet * 2
        } else {
            match self.total.cmp(&dealer_hand.total) {
                Ordering::Less => 0,
                Ordering::Equal => bet,
                Ordering::Greater => bet * 2,
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
    fn shuffle(&mut self);

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
    dist: WeightedIndex<u8>,
    remaining: [u8; 52],
}

impl MultiDeck {
    fn with_size(size: u8) -> Self {
        let remaining = [size; 52];
        let dist = WeightedIndex::new(remaining).unwrap();
        MultiDeck { size, dist, remaining }
    }
}

impl Shoe for MultiDeck {
    fn draw(&mut self) -> Card {
        let ordinal = self.dist.sample(&mut thread_rng());
        let new_weight = self.remaining[ordinal] - 1;
        self.dist.update_weights(&[(ordinal, &new_weight)]).unwrap();
        Card::from_ordinal(ordinal)
    }
    fn shuffle(&mut self) {
        self.remaining = [self.size; 52];
        self.dist = WeightedIndex::new(self.remaining).unwrap();
    }
}

struct InfiniteDeck;

impl Shoe for InfiniteDeck {
    fn draw(&mut self) -> Card {
        Card::random()
    }
    fn shuffle(&mut self) {
        // Do nothing
    }
}

pub fn play(config: Blackjack) {
    println!("Welcome to Blackjack!");
    let mut deck: Box<dyn Shoe> = match config.decks {
        Some(decks) => Box::new(MultiDeck::with_size(decks)),
        None => Box::new(InfiniteDeck),
    };
    let mut player_chips = config.chips;
    'game: while let Some(mut bet) = place_bet(player_chips, config.max_bet, config.min_bet) {
        player_chips -= bet;
        println!("You bet {bet} chips. You have {player_chips} chips remaining.");

        let player_card = deck.draw_player();
        let dealer_card = deck.draw_dealer(false);

        let player_hand = player_card + deck.draw_player();

        // TODO: Remove this pre-check, this can be handled later with the normal logic
        if player_hand.is_21() {
            let dealer_hand = dealer_card + deck.draw_dealer(false);
            if dealer_hand.is_21() {
                reveal_dealer_hand(&dealer_hand);
                println!("The dealer also has blackjack!");
                println!("It's a push!");
                player_chips += bet;
                continue 'game;
            }
            println!("You win {} chips!", bet * 5 / 2);
            player_chips += bet * 5 / 2;
            continue 'game;
        }

        let mut dealer_hand = dealer_card + deck.draw_dealer(true);

        let dealer_showing = dealer_hand.cards[0].rank.value();
        if dealer_showing == 10 || dealer_showing == 11 {
            println!("The dealer checks their cards...");
        }

        // TODO: Can probably handle this with the normal logic too
        if dealer_hand.is_21() {
            reveal_dealer_hand(&dealer_hand);
            println!("The dealer has blackjack! You lose!");
            continue 'game;
        }

        let mut hands = vec![player_hand];
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

        // Probably unnecessary after the loop, but this is a good sanity check
        // TODO: Remove this and still feel confident
        assert!(hands.iter().all(|hand| hand.done));

        reveal_dealer_hand(&dealer_hand);
        println!("The dealer has {}.", dealer_hand.total);

        if hands.iter().any(|hand| !hand.is_bust()) {
            // At least one hand is not bust, so the dealer must play
            while !dealer_hand.done {
                dealer_hand += deck.draw_dealer(false);

                if dealer_hand.total < 17 {
                    continue;
                }
                match &config.dealer {
                    StandOn::Soft17 => dealer_hand.done = true,
                    StandOn::Hard17 => if !dealer_hand.soft || dealer_hand.total > 17 {
                        dealer_hand.done = true;
                    },
                }
            }
        }

        let chips_won: u32 = hands
            .into_iter()
            .map(|hand| hand.winnings_against(&dealer_hand, bet))
            .sum();

        match chips_won {
            0 => println!("You lose!"),
            chips if chips < bet => println!("You make back {chips} chips."),
            chips if chips == bet => println!("You push!"),
            chips => println!("You win {chips} chips!"),
        }
        player_chips += chips_won;
        // FIXME: Any place with continue 'game will skip this
        if config.shuffle == ShuffleStrategy::EveryRound {
            deck.shuffle();
        }
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

fn reveal_dealer_hand(hand: &Hand) {
    thread::sleep(Duration::from_secs(1));
    println!("The dealer reveals a {}.", hand.cards[1]);
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
    print!("{}\t\t{}\t\t", Action::Hit, Action::Stand);
    if can_double_bet && hand.is_two_cards() {
        print!("{}\t\t", Action::DoubleDown);
    }
    if can_double_bet && hand.is_pair() {
        print!("{}\t\t", Action::Split);
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
