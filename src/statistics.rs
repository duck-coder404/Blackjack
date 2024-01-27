use std::fmt::Display;
use crate::card::hand::Status;
use crate::game::EndTurn;

#[derive(Default)]
pub struct Statistics {
    turns_played: usize,
    hands_played: usize,
    total_bet: usize,
    wins: usize,
    pushes: usize,
    losses: usize,
    blackjacks: usize,
    busts: usize,
    dealer_blackjacks: usize,
    dealer_busts: usize,
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            turns_played: 0,
            hands_played: 0,
            wins: 0,
            pushes: 0,
            losses: 0,
            total_bet: 0,
            blackjacks: 0,
            busts: 0,
            dealer_blackjacks: 0,
            dealer_busts: 0,
        }
    }

    pub fn update(&mut self, turn: &EndTurn) {
        self.turns_played += 1;
        self.hands_played += turn.player_hands.len();
        self.wins += turn.player_hands.iter().filter(|hand| hand.winnings > hand.bet).count();
        self.pushes += turn.player_hands.iter().filter(|hand| hand.winnings == hand.bet).count();
        self.losses += turn.player_hands.iter().filter(|hand| hand.winnings < hand.bet).count();
        self.total_bet += turn.player_hands.iter().map(|hand| hand.bet as usize).sum::<usize>();
        self.blackjacks += turn.player_hands.iter().filter(|hand| hand.status == Status::Blackjack).count();
        self.busts += turn.player_hands.iter().filter(|hand| hand.status == Status::Bust).count();
        self.dealer_blackjacks += usize::from(turn.dealer_hand.status == Status::Blackjack);
        self.dealer_busts += usize::from(turn.dealer_hand.status == Status::Bust);
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn pct(n: usize, d: usize) -> String {
            if d == 0 {
                "0.0".to_string()
            } else {
                format!("{:.2}", n as f64 / d as f64 * 100.0)
            }
        }
        fn div(n: usize, d: usize) -> String {
            if d == 0 {
                "0.0".to_string()
            } else {
                format!("{:.2}", n as f64 / d as f64)
            }
        }

        writeln!(f, "{{")?;
        writeln!(f, "  Turns Played: {}", self.turns_played)?;
        writeln!(f, "  Hands Played: {}", self.hands_played)?;
        writeln!(f, "  Total Bet: {} Chips", self.total_bet)?;
        writeln!(f, "  Average Bet: {} Chips", div(self.total_bet, self.hands_played))?;
        writeln!(f, "  Wins: {} ({}%)", self.wins, pct(self.wins, self.hands_played))?;
        writeln!(f, "  Pushes: {} ({}%)", self.pushes, pct(self.pushes, self.hands_played))?;
        writeln!(f, "  Losses: {} ({}%)", self.losses, pct(self.losses, self.hands_played))?;
        writeln!(f, "  Blackjacks: {} ({}%)", self.blackjacks, pct(self.blackjacks, self.hands_played))?;
        writeln!(f, "  Busts: {} ({}%)", self.busts, pct(self.busts, self.hands_played))?;
        writeln!(f, "  Dealer Blackjacks: {} ({}%)", self.dealer_blackjacks, pct(self.dealer_blackjacks, self.hands_played))?;
        writeln!(f, "  Dealer Busts: {} ({}%)", self.dealer_busts, pct(self.dealer_busts, self.hands_played))?;
        write!(f, "}}")?;

        Ok(())
    }
}