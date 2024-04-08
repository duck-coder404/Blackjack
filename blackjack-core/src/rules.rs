#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DealerSoft17Action {
    #[default]
    Stand,
    Hit,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BlackjackPayout {
    #[default]
    ThreeToTwo,
    SixToFive
}

#[derive(Debug, Clone)]
pub struct Rules {
    pub max_bet: Option<u32>,
    pub min_bet: Option<u32>,
    pub blackjack_payout: BlackjackPayout,
    pub dealer_soft_17: DealerSoft17Action,
    pub offer_insurance: bool,
    pub offer_early_surrender: bool,
    pub offer_late_surrender: bool,
    pub max_splits: Option<u8>,
    pub double_after_split: bool,
    pub split_aces: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            max_bet: None,
            min_bet: Some(100),
            blackjack_payout: BlackjackPayout::ThreeToTwo,
            dealer_soft_17: DealerSoft17Action::Stand,
            offer_insurance: false,
            offer_early_surrender: false,
            offer_late_surrender: true,
            max_splits: Some(5),
            double_after_split: true,
            split_aces: true,
        }
    }
}