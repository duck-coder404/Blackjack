use std::io;
use std::fmt::Write;

use blackjack_core::card::hand::PlayerHand;
use blackjack_core::game::Table;

use crate::input::HandAction;

pub fn place_bet_or_quit(game: &Table, chips: u32) -> Option<u32> {
    println!(
        "You have {chips} chips. How many chips would you like to bet? Type \"stop\" to quit."
    );
    let mut input = String::new();
    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let trimmed = input.trim();
        if trimmed == "stop" {
            return None;
        }
        match trimmed.parse() {
            Ok(0) => println!("You must bet at least 1 chip!"),
            Ok(bet) if bet > chips => println!("You don't have enough chips!"),
            Ok(bet) => match (game.rules.max_bet, game.rules.min_bet) {
                (Some(max), _) if bet > max => println!("You cannot bet more than {max} chips!"),
                (_, Some(min)) if bet < min => println!("You cannot bet fewer than {min} chips!"),
                _ => return Some(bet),
            },
            Err(_) => println!("Please enter a number!"),
        }
        input.clear();
    }
}

pub fn surrender_early() -> bool {
    println!("Would you like to surrender before the dealer checks for blackjack? (y/n)");
    let mut input = String::new();
    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match input.trim() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please enter y or n!"),
        }
        input.clear();
    }
}

pub fn offer_insurance(max_bet: u32) -> u32 {
    println!("Would you like to place an insurance bet? Enter your bet or 0 to decline.");
    let mut input = String::new();
    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match input.trim().parse() {
            Ok(0) => return 0,
            Ok(bet) if bet > max_bet => {
                println!("You cannot bet more than half your original bet!")
            }
            Ok(bet) => {
                println!("You place an insurance bet of {bet} chips.");
                return bet;
            }
            Err(_) => println!("Please enter a number!"),
        }
        input.clear();
    }
}

/// Prompts the player to make a move
/// Which actions are available depends on the number of cards in the hand,
/// whether the hand is a pair, and whether the player has enough chips to double their bet
pub fn get_hand_action(
    table: &Table,
    player_hand: &PlayerHand,
    chips: u32,
) -> HandAction {
    let mut allowed_moves = Vec::with_capacity(5);
    allowed_moves.push("Hit (h)");
    allowed_moves.push("Stand (s)");
    if table.is_allowed_to_double(player_hand) {
        allowed_moves.push("Double (d)");
    }
    if table.is_allowed_to_split(player_hand) {
        allowed_moves.push("Split (p)");
    }
    if table.is_allowed_to_surrender(player_hand) {
        allowed_moves.push("Surrender (u)");
    }
    let allowed_moves: String =
        allowed_moves
            .into_iter()
            .fold(String::with_capacity(75), |mut acc, s| {
                let _ = write!(acc, "{:15}", s).unwrap();
                acc
            });
    println!("What would you like to do?\n{}", allowed_moves);

    let mut input = String::new();
    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match parse_hand_action(input.trim()) {
            Ok(action) => match action {
                HandAction::Double if player_hand.size() != 2 => {
                    println!("You can only double down on your first two cards!")
                }
                HandAction::Double if chips < player_hand.bet => {
                    println!("You don't have enough chips to double down!")
                }
                HandAction::Double if player_hand.splits != 0 && !table.rules.double_after_split => {
                    println!("You can't double down after splitting!")
                }
                HandAction::Split if !player_hand.is_pair() => println!("You can only split a pair!"),
                HandAction::Split if chips < player_hand.bet => {
                    println!("You don't have enough chips to split!")
                }
                HandAction::Split if table.rules.max_splits.map_or(false, |max| player_hand.splits >= max) => println!("You can't split again!"),
                HandAction::Split if player_hand.value.soft && !table.rules.split_aces => println!("You can't split aces!"),
                HandAction::Surrender if player_hand.size() != 2 || !table.rules.offer_late_surrender => println!("You can't surrender!"),
                action => return action,
            },
            Err(_) => println!("Please enter a valid action!"),
        };
        input.clear();
    }
}

fn parse_hand_action(s: &str) -> Result<HandAction, ()> {
    match s {
        "s" | "stand" => Ok(HandAction::Stand),
        "h" | "hit" => Ok(HandAction::Hit),
        "d" | "double" => Ok(HandAction::Double),
        "p" | "split" => Ok(HandAction::Split),
        "u" | "surrender" => Ok(HandAction::Surrender),
        _ => Err(()),
    }
}
