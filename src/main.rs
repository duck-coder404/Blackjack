use clap::{Parser, ValueEnum};

mod game;
mod card;

#[derive(Debug, Parser)]
#[command(author, about, version)]
pub struct Configuration {
    /// Number of decks to use. Defaults to infinite.
    #[arg(long, short)]
    pub decks: Option<u8>,
    /// When to shuffle the deck. This has no effect if `decks` is not set.
    #[arg(long, short, value_enum, default_value_t = ShuffleStrategy::Continuous)]
    pub shuffle: ShuffleStrategy,
    /// Dealer strategy on soft 17s.
    #[arg(long, value_enum, default_value_t = Soft17::Stand)]
    pub soft17: Soft17,
    /// Blackjack payout. Defaults to `ThreeToTwo`.
    #[arg(long, short, value_enum, default_value_t = BlackjackPayout::ThreeToTwo)]
    pub payout: BlackjackPayout,
    /// Number of chips to start with. Defaults to 1000.
    #[arg(short, long, default_value_t = 1000)]
    pub chips: u32,
    /// Max bet allowed. Defaults to None.
    #[arg(long)]
    pub max_bet: Option<u32>,
    /// Min bet allowed. Defaults to None.
    #[arg(long)]
    pub min_bet: Option<u32>,
    /// Whether to allow surrendering. Defaults to false.
    #[arg(long, default_value_t = false)]
    pub surrender: bool, // TODO: Implement
    /// Whether to offer insurance. Defaults to false.
    #[arg(long, short, default_value_t = false)]
    pub insurance: bool, // TODO: Implement
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum ShuffleStrategy {
    /// Shuffle the shoe after every round. Worst for counting cards.
    Continuous,
    /// Shuffle when the shoe is a quarter empty.
    QuarterShoe,
    /// Shuffle when the shoe is half empty.
    HalfShoe,
    /// Shuffle when the shoe is three quarters empty.
    ThreeQuartersShoe,
    /// Shuffle when the shoe is empty. Best for counting cards.
    EmptyShoe,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum Soft17 {
    /// Dealer stands on soft 17.
    Stand,
    /// Dealer hits on soft 17.
    Hit,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum BlackjackPayout {
    /// Blackjack pays 3:2.
    ThreeToTwo,
    /// Blackjack pays 6:5.
    SixToFive,
}

fn main() {
    let config = Configuration::parse();
    println!("Starting {:#?}\n", config);
    game::play(config);
}
