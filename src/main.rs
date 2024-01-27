use clap::Parser;
use crate::game::Game;
use crate::input::basic::BasicStrategy;
use crate::input::io::IO;
use crate::input::Player;

mod card;
mod game;
mod input;
mod statistics;

#[derive(Debug, Parser)]
#[command(author, about, version)]
pub struct Configuration {
    /// Number of decks to use.
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=255))]
    pub decks: u8,
    /// Proportion of the shoe to play before reshuffling.
    #[arg(short, long, default_value_t = 0.0, value_parser = parse_float_between_0_and_1)]
    pub penetration: f32,
    /// Whether the dealer should hit on soft 17s.
    #[arg(long, default_value_t = false)]
    pub soft_17_hit: bool,
    /// Whether blackjack pays 6:5 instead of 3:2.
    #[arg(long, default_value_t = false)]
    pub six_to_five: bool,
    /// Whether to allow early surrendering.
    #[arg(long, short, default_value_t = false)]
    pub early_surrender: bool,
    /// Whether to allow late surrendering.
    #[arg(long, short, default_value_t = false)]
    pub late_surrender: bool,
    /// Whether to allow splitting aces.
    #[arg(long, default_value_t = true)]
    pub split_aces: bool,
    /// Whether to allow double after split.
    #[arg(long, default_value_t = true)]
    pub double_after_split: bool,
    /// Maximum number of splits allowed.
    #[arg(long)]
    pub max_splits: Option<u8>,
    /// Whether to offer insurance.
    #[arg(short, long, default_value_t = false)]
    pub insurance: bool,
    /// Number of chips to start with. Defaults to 1000.
    #[arg(short, long, default_value_t = 1000)]
    pub chips: u32,
    /// Maximum bet allowed.
    #[arg(long)]
    pub max_bet: Option<u32>,
    /// Minimum bet allowed.
    #[arg(long)]
    pub min_bet: Option<u32>,
    /// Enable simulation mode.
    #[arg(long, short)]
    pub simulate: Option<u32>,
}

fn parse_float_between_0_and_1(s: &str) -> Result<f32, String> {
    let f = s
        .parse::<f32>()
        .map_err(|_| format!("{s} is not a valid float"))?;
    if (0.0..=1.0).contains(&f) {
        Ok(f)
    } else {
        Err(format!("{f} is not between 0 and 1"))
    }
}

fn main() {
    let config = Configuration::parse();
    println!("Using {config:#?}\n");
    assert!(config.chips >= config.min_bet.unwrap_or(1), "You don't have enough chips to play!");
    if let (Some(max), Some(min)) = (config.max_bet, config.min_bet) {
        assert!(max >= min, "Max bet cannot be less than min bet!");
    }

    let mut player = match config.simulate {
        Some(turns) => Player::new(config.chips, BasicStrategy::new(turns, config.min_bet.unwrap_or(config.chips / 100))),
        None => Player::new(config.chips, IO),
    };
    Game::new(&config).play(&mut player);
}
