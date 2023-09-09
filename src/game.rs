use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::Duration;
use std::{io, thread};

use crate::Configuration;
use crate::card::hand::{DealerHand, Hand, PlayerHand};
use crate::card::shoe::Shoe;

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

        if player_hands.iter().any(|hand| hand.is_stood()) {
            // At least one hand was played and stood on, so the dealer must finish their hand
            while !dealer_hand.is_over() {
                dealer_hand += deck.draw();
            }
        }

        // At this point, all hands are done
        // For each hand, determine the result and payout
        let dealer_status = dealer_hand.status();
        let chips_won: u32 = player_hands
            .into_iter()
            .map(|hand| hand.winnings(&dealer_status, &config.payout))
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
    while let Some(hand) = hands.iter_mut().find(|hand| !hand.is_over()) {
        pause();
        println!(
            "What would you like to do? ({} against {})",
            hand, dealer_hand.showing()
        );
        match player_action(hand, *player_chips >= hand.bet()) {
            Action::Stand => {
                println!("You stand!");
                hand.stand();
            }
            Action::Hit => {
                println!("You hit!");
                *hand += deck.draw();
            }
            Action::DoubleDown => {
                println!("You double and put another {} chips down!", hand.bet());
                *player_chips -= hand.bet(); // The player pays another equal bet
                hand.double(deck.draw());
            }
            Action::Split => {
                println!("You split your hand and put another {} chips down!", hand.bet());
                *player_chips -= hand.bet(); // The player pays another equal bet for the new hand
                let mut new_hand = PlayerHand::split(hand);
                *hand += deck.draw();
                new_hand += deck.draw();
                hands.push(new_hand);
            }
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
