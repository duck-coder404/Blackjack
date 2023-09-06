use std::fmt::{self, Display, Formatter};
use std::ops::{Add, AddAssign};
use std::str::FromStr;
use std::time::Duration;
use std::{io, thread};

use rand::Rng;

#[derive(Debug)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..=3) {
            0 => Suit::Clubs,
            1 => Suit::Diamonds,
            2 => Suit::Hearts,
            3 => Suit::Spades,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
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
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..=12) {
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
            _ => unreachable!(),
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

struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    fn random() -> Self {
        let rank = Rank::random();
        let suit = Suit::random();
        Card { rank, suit }
    }
    #[allow(dead_code)]
    fn with_rank(rank: Rank) -> Self {
        let suit = Suit::random();
        Card { rank, suit }
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
                if rhs.0.rank == Rank::Ace {
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
    done: bool,
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
                std::cmp::Ordering::Less => 0,
                std::cmp::Ordering::Equal => bet,
                std::cmp::Ordering::Greater => bet * 2,
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

fn main() {
    let mut player_chips = 1000;
    'game: while let Some(mut bet) = place_bet(player_chips) {
        player_chips -= bet;
        println!("You bet {bet} chips. You have {player_chips} chips remaining.");

        let player_card = draw_player_card();
        let dealer_card = draw_dealer_card(false);

        let player_hand = player_card + draw_player_card();

        if player_hand.is_21() {
            let dealer_hand = dealer_card + draw_dealer_card(false);
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

        let mut dealer_hand = dealer_card + draw_dealer_card(true);

        let dealer_showing = dealer_hand.cards[0].rank.value();
        if dealer_showing == 10 || dealer_showing == 11 {
            println!("The dealer checks their cards...");
        }

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
                    *hand += draw_player_card();
                }
                Action::DoubleDown => {
                    println!("You double and put another {bet} chips down!");
                    player_chips -= bet;
                    bet *= 2; // Double the bet for this hand
                    *hand += draw_player_card();
                    hand.done = true;
                }
                Action::Split => {
                    println!("You split your hand into two and put another {bet} chips down!");
                    player_chips -= bet; // Do not double the bet, each hand is worth the original bet
                    let right = hand.split_and_replace(draw_player_card());
                    let right_hand = right + draw_player_card();
                    hands.push(right_hand);
                }
            }
        }

        assert!(hands.iter().all(|hand| hand.done));

        reveal_dealer_hand(&dealer_hand);
        println!("The dealer has {}.", dealer_hand.total);

        if hands.iter().any(|hand| !hand.is_bust()) {
            // At least one hand is not bust
            while !dealer_hand.done {
                dealer_hand += draw_dealer_card(false);
                if dealer_hand.total >= 17 {
                    dealer_hand.done = true;
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
    }
    println!("You finished with {player_chips} chips.");
    println!("Goodbye!");
    thread::sleep(Duration::from_secs(2));
}

fn place_bet(chips: u32) -> Option<u32> {
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
            Ok(bet) if bet <= chips => break Some(bet),
            Ok(_) => println!("You don't have enough chips! You only have {chips} chips."),
            Err(_) => println!("Please enter a number!"),
        }
        bet.clear();
    }
}

fn draw_player_card() -> PlayerCard {
    thread::sleep(Duration::from_secs(1));
    let card = Card::random();
    println!("You draw {card}.");
    PlayerCard(card)
}

fn draw_dealer_card(hidden: bool) -> DealerCard {
    thread::sleep(Duration::from_secs(1));
    let card = Card::random();
    if hidden {
        println!("The dealer draws a hidden card.");
    } else {
        println!("The dealer draws {card}.");
    }
    DealerCard(card)
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
    println!("{}", Action::Hit);
    println!("{}", Action::Stand);
    if can_double_bet && hand.is_two_cards() {
        println!("{}", Action::DoubleDown);
    }
    if can_double_bet && hand.is_pair() {
        println!("{}", Action::Split);
    }
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
