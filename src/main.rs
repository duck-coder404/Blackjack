use clap::{Parser, ValueEnum};

mod game;
mod card;
mod io;

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
    #[arg(long, value_enum, default_value_t = Surrender::None)]
    pub surrender: Surrender,
    /// Whether to offer insurance. Defaults to false.
    #[arg(long, short, default_value_t = false)]
    pub insurance: bool, // TODO: Implement
}

/// Casinos have different regulations on when to shuffle.
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum ShuffleStrategy {
    /// Shuffle the shoe after every round, so all cards are always in play.
    /// This is most common, and made possible by CSMs (continuous shuffling machines).
    /// It also makes counting cards impossible.
    Continuous,
    /// Shuffle when the shoe is a quarter empty.
    QuarterShoe,
    /// Shuffle when the shoe is half empty.
    HalfShoe,
    /// Shuffle when the shoe is three quarters empty.
    ThreeQuartersShoe,
    /// Shuffle when the shoe is empty.
    /// No casinos would do this, but it is possible.
    /// Best for counting cards.
    EmptyShoe,
}

/// Casinos have different regulations on how the dealer should play on soft 17s.
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum Soft17 {
    /// Dealer stands on soft 17.
    /// This is more common in casinos and offers the player a slight advantage.
    Stand,
    /// Dealer hits on soft 17.
    Hit,
}

/// Casinos have different regulations on blackjack payouts.
#[derive(Debug, Clone, ValueEnum)]
pub enum BlackjackPayout {
    /// Blackjack pays 3:2.
    /// This is more common in casinos and offers the player a moderate advantage.
    ThreeToTwo,
    /// Blackjack pays 6:5.
    SixToFive,
}

/// Surrendering allows the player to forfeit their hand and receive half their bet back,
/// in case they think they have a low chance of winning.
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum Surrender {
    /// No surrendering allowed.
    None,
    /// Surrendering allowed before the dealer checks their hole card for blackjack.
    /// Offers protection against dealer blackjack.
    Early,
    /// Surrendering allowed when the player is choosing their move.
    /// This is the most common form of surrendering.
    Late,
    /// Both early and late surrendering allowed.
    Both,
}

fn main() {
    let config = Configuration::parse();
    println!("Starting {:#?}\n", config);
    game::play(config);
}
