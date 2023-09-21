use clap::{Parser, ValueEnum};

mod card;
mod game;
mod io;

#[derive(Debug, Parser)]
#[command(author, about, version)]
pub struct Configuration {
    /// Number of decks to use.
    #[arg(long, short, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=255))]
    pub decks: u8,
    /// Proportion of cards to play before shuffling.
    /// 0 => every hand, 0.5 => half shoe, 1 => entire shoe.
    #[arg(long, short, default_value_t = 0.0, value_parser = parse_float_between_0_and_1)]
    pub penetration: f32,
    /// Dealer strategy on soft 17s.
    #[arg(long, value_enum, default_value_t = Soft17::Stand)]
    pub soft17: Soft17,
    /// Whether blackjack pays 3:2 or 6:5.
    #[arg(long, short, value_enum, default_value_t = BlackjackPayout::ThreeToTwo)]
    pub blackjack_payout: BlackjackPayout,
    /// Number of chips to start with. Defaults to 1000.
    #[arg(short, long, default_value_t = 1000)]
    pub chips: u32,
    /// Max bet allowed.
    #[arg(long)]
    pub max_bet: Option<u32>,
    /// Min bet allowed.
    #[arg(long)]
    pub min_bet: Option<u32>,
    /// Whether to allow surrendering.
    #[arg(long, value_enum, default_value_t = SurrenderAllowed::None)]
    pub surrender: SurrenderAllowed,
    /// Whether to offer insurance.
    #[arg(long, short, default_value_t = false)]
    pub insurance: bool,
}

fn parse_float_between_0_and_1(s: &str) -> Result<f32, String> {
    let f = s
        .parse::<f32>()
        .map_err(|_| format!("{} is not a valid float", s))?;
    if (0.0..=1.0).contains(&f) {
        Ok(f)
    } else {
        Err(format!("{} is not between 0 and 1", f))
    }
}

/// Casinos have different regulations on how the dealer should play on soft 17s.
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum Soft17 {
    /// Dealer stands on soft 17.
    Stand,
    /// Dealer hits on soft 17.
    Hit,
}

/// Casinos have different regulations on blackjack payouts.
#[derive(Debug, Clone, ValueEnum)]
pub enum BlackjackPayout {
    /// Blackjack pays 3:2.
    ThreeToTwo,
    /// Blackjack pays 6:5.
    SixToFive,
}

/// Surrendering allows the player to forfeit their hand and receive half their bet back,
/// in case they think they have a low chance of winning.
/// Some casinos offer this, while others don't.
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum SurrenderAllowed {
    /// No surrendering allowed.
    None,
    /// Surrendering allowed before the dealer checks their hole card for blackjack.
    Early,
    /// Surrendering allowed when the player is choosing their move.
    Late,
    /// Both early and late surrendering allowed.
    Both,
}

fn main() {
    let config = Configuration::parse();
    println!("Using {:#?}\n", config);
    game::play(config);
}
