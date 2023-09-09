use std::fmt::{Display, Formatter};
use std::{fmt, io};
use std::str::FromStr;

/// Prompts the player to place a bet or quit
/// Returns Some(bet) if the player wants to bet bet chips
/// Returns None if the player wants to quit
pub(crate) fn place_bet(chips: u32, max_bet: Option<u32>, min_bet: Option<u32>) -> Option<u32> {
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
}

/// Prompts the player to make a move
/// Which actions are available depends on the number of cards in the hand,
/// whether the hand is a pair, and whether the player has enough chips to double their bet
pub(crate) fn make_move(cards: usize, is_pair: bool, can_double_bet: bool) -> Action {
    let mut action = format!("{}\t\t{}", Action::Hit, Action::Stand);
    if can_double_bet && cards == 2 {
        action.push_str(&format!("\t{}", Action::DoubleDown));
    }
    if can_double_bet && is_pair {
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
        action.clear();
    }
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
