use std::fmt::{Display, Formatter};
use std::{fmt, io};
use std::str::FromStr;
use crate::Surrender;

/// Prompts the player to place a bet or quit
/// Returns Some(bet) if the player wants to bet bet chips
/// Returns None if the player wants to quit
pub(crate) fn place_bet(
    chips: u32,
    max_bet: Option<u32>,
    min_bet: Option<u32>,
) -> Option<u32> {
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
pub(crate) enum Action {
    Stand,
    Hit,
    DoubleDown,
    Split,
    Surrender,
}

macro_rules! offer {
    ($string:expr, $action:ident) => {
        let formatted = format!("{:15}", format!("{}", Action::$action));
        $string.push_str(&formatted);
    }
}

/// Prompts the player to make a move
/// Which actions are available depends on the number of cards in the hand,
/// whether the hand is a pair, and whether the player has enough chips to double their bet
pub(crate) fn make_move(
    cards: usize,
    is_pair: bool,
    can_double_bet: bool,
    surrender: &Surrender,
) -> Action {
    let mut offering = String::new();
    offer!(offering, Hit);
    offer!(offering, Stand);
    if can_double_bet && cards == 2 {
        offer!(offering, DoubleDown);
    }
    if can_double_bet && is_pair {
        offer!(offering, Split);
    }
    if surrender == &Surrender::Late || surrender == &Surrender::Both {
        offer!(offering, Surrender);
    }
    println!("{}", offering);
    offering.clear(); // Reuse the string
    loop {
        io::stdin()
            .read_line(&mut offering)
            .expect("Failed to read input");
        match offering.trim().parse() {
            Ok(action) => match action {
                Action::DoubleDown if cards != 2 => {
                    println!("You can only double down on your first two cards!")
                }
                Action::DoubleDown if !can_double_bet => {
                    println!("You don't have enough chips to double down!")
                }
                Action::Split if !is_pair => println!("You can only split a pair!"),
                Action::Split if !can_double_bet => {
                    println!("You don't have enough chips to split!")
                }
                action => return action,
            },
            Err(_) => println!("Please enter a valid action!"),
        };
        offering.clear();
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Action::Stand => write!(f, "Stand (s)"),
            Action::Hit => write!(f, "Hit (h)"),
            Action::DoubleDown => write!(f, "Double (d)"),
            Action::Split => write!(f, "Split (p)"),
            Action::Surrender => write!(f, "Surrender (u)"),
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
            "u" => Ok(Action::Surrender),
            "surrender" => Ok(Action::Surrender),
            _ => Err(()),
        }
    }
}

pub(crate) fn offer_early_surrender() -> bool {
    println!("Would you like to surrender before the dealer checks for blackjack? (y/n)");
    let mut input = String::new();
    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match input.trim() {
            "y" => return true,
            "yes" => return true,
            "n" => return false,
            "no" => return false,
            _ => println!("Please enter y or n!"),
        }
        input.clear();
    }
}
