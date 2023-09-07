use clap::{Parser, ValueEnum};

mod game;

#[derive(Debug, Parser)]
#[command(author, about, version)]
pub struct Blackjack {
    /// Number of decks to use. Defaults to infinite.
    #[arg(long, short)]
    pub decks: Option<u8>,
    /// When to shuffle the deck. Defaults to `EveryRound`.
    #[arg(long, short, value_enum, default_value_t = ShuffleStrategy::EveryRound)]
    pub shuffle: ShuffleStrategy,
    /// Dealer strategy. Defaults to `Soft17`.
    #[arg(long, value_enum, default_value_t = StandOn::Soft17)]
    pub dealer: StandOn,
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
    #[arg(long, default_value_t = false)]
    pub insurance: bool, // TODO: Implement
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum ShuffleStrategy {
    /// Shuffle the deck after every round.
    EveryRound,
    // TODO: Implement more strategies
    // (e.g. after a certain number of rounds, after a certain number of cards, etc.)
}

#[derive(Debug, Clone, ValueEnum)]
pub enum StandOn {
    /// Dealer stands on soft 17.
    Soft17,
    /// Dealer hits on soft 17.
    Hard17,
}

fn main() {
    let config = Blackjack::parse();
    println!("Starting {:#?}", config);
    game::play(config);
}
