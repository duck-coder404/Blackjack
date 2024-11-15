use crate::card::Card;

pub enum DeckError {
    NotEnoughCards,
    TooManyCards,
    InvalidCard,
}

pub struct Deck {
    cards: [[u8; 13]; 4], // 4 suits, 13 ranks
}

impl Deck {
    pub fn new() -> Self {
        Self {
            cards: [[1; 13]; 4],
        }
    }
    pub fn times(&mut self, n: u8) {
        // multiply all cards by n
        for suit in self.cards.iter_mut() {
            for rank in suit.iter_mut() {
                *rank = rank.checked_mul(n).unwrap(); // todo: handle overflow
            }
        }
    }
    pub fn count(&self, card: Card) -> u8 {
        self.cards[card.suit as usize][card.rank as usize]
    }
    pub fn add(&mut self, card: Card) -> Result<(), DeckError> {
        let amount = &mut self.cards[card.suit as usize][card.rank as usize];
        *amount = amount.checked_add(1).ok_or(DeckError::TooManyCards)?;
        Ok(())
    }
    pub fn remove(&mut self, card: Card) -> Result<(), DeckError> {
        let amount = &mut self.cards[card.suit as usize][card.rank as usize];
        *amount = amount.checked_sub(1).ok_or(DeckError::NotEnoughCards)?;
        Ok(())
    }
}