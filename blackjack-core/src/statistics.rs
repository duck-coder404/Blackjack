use std::cmp::Ordering;
use std::fmt::Display;
use crate::card::hand::{DealerHand, FinishedTurn, PlayerHand, Status};

#[derive(Debug, Default)]
pub struct Statistics {
    rounds_played: usize,
    turns_played: usize,
    hands_played: usize,
    total_bet: usize,
    total_won: usize,
    wins: usize,
    pushes: usize,
    losses: usize,
    blackjacks: usize,
    busts: usize,
    dealer_blackjacks: usize,
    dealer_busts: usize,
}

impl Statistics {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rounds_played: 0,
            turns_played: 0,
            hands_played: 0,
            total_bet: 0,
            total_won: 0,
            wins: 0,
            pushes: 0,
            losses: 0,
            blackjacks: 0,
            busts: 0,
            dealer_blackjacks: 0,
            dealer_busts: 0,
        }
    }

    /// Update the statistics with the results of a round of blackjack.
    pub fn update(&mut self, player_turns: Vec<FinishedTurn>, dealer_hand: DealerHand) {
        self.rounds_played += 1;
        self.turns_played += player_turns.len();
        self.hands_played += player_turns.iter().map(|t| t.hands.len()).sum::<usize>();
        for hand in player_turns.iter().flat_map(|t| &t.hands) {
            match hand.status {
                Status::Blackjack => self.blackjacks += 1,
                Status::Bust => self.busts += 1,
                _ => {},
            }
            // TODO: Handle winning statistics
            // match hand.winnings.cmp(&hand.bet) {
            //     Ordering::Greater => self.wins += 1,
            //     Ordering::Equal => self.pushes += 1,
            //     Ordering::Less => self.losses += 1,
            // }
            self.total_bet = self.total_bet.saturating_add(hand.bet as usize);
            // self.total_won = self.total_won.saturating_add(hand.winnings as usize);
        }
        match dealer_hand.status {
            Status::Blackjack => self.dealer_blackjacks += 1,
            Status::Bust => self.dealer_busts += 1,
            _ => {},
        }
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn pct(num: usize, div: usize) -> String {
            if div == 0 {
                "0.0".to_string()
            } else {
                format!("{:.2}", num as f64 / div as f64 * 100.0)
            }
        }
        fn div(num: usize, div: usize) -> String {
            if div == 0 {
                "0.0".to_string()
            } else {
                format!("{:.2}", num as f64 / div as f64)
            }
        }

        writeln!(f, "Turns Played: {}", self.turns_played)?;
        writeln!(f, "Hands Played: {}", self.hands_played)?;
        writeln!(f, "Total Bet: {} Chips", self.total_bet)?;
        writeln!(f, "Average Bet: {} Chips", div(self.total_bet, self.hands_played))?;
        writeln!(f, "Total Won: {} Chips", self.total_won)?;
        writeln!(f, "Average Win: {} Chips", div(self.total_won, self.hands_played))?;
        writeln!(f, "Wins: {} ({}%)", self.wins, pct(self.wins, self.hands_played))?;
        writeln!(f, "Pushes: {} ({}%)", self.pushes, pct(self.pushes, self.hands_played))?;
        writeln!(f, "Losses: {} ({}%)", self.losses, pct(self.losses, self.hands_played))?;
        writeln!(f, "Blackjacks: {} ({}%)", self.blackjacks, pct(self.blackjacks, self.hands_played))?;
        writeln!(f, "Busts: {} ({}%)", self.busts, pct(self.busts, self.hands_played))?;
        writeln!(f, "Dealer Blackjacks: {} ({}%)", self.dealer_blackjacks, pct(self.dealer_blackjacks, self.hands_played))?;
        writeln!(f, "Dealer Busts: {} ({}%)", self.dealer_busts, pct(self.dealer_busts, self.hands_played))?;

        Ok(())
    }
}