//! Blackjack table rules.

/// The action the dealer takes on a soft 17.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DealerSoft17Action {
    Stand,
    Hit,
}

/// The payout for a blackjack, either 3:2 or 6:5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlackjackPayout {
    ThreeToTwo,
    SixToFive,
}

/// Blackjack table rules.
#[derive(Debug, Clone)]
pub struct Rules {
    /// The maximum bet allowed, if any.
    pub max_bet: Option<u32>,
    /// The minimum bet allowed, if any.
    pub min_bet: Option<u32>,
    /// The payout for a blackjack.
    pub blackjack_payout: BlackjackPayout,
    /// The action the dealer takes on a soft 17.
    pub dealer_soft_17: DealerSoft17Action,
    /// Whether to offer insurance.
    pub insurance: bool,
    /// Whether players are allowed to surrender before the dealer checks for blackjack.
    pub early_surrender: bool,
    /// Whether players are allowed to surrender after the dealer checks for blackjack.
    pub late_surrender: bool,
    /// The maximum number of times a player can split a hand.
    pub max_splits: Option<u8>,
    /// Whether players can double down on a split hand.
    pub double_after_split: bool,
    /// Whether players can split aces.
    pub split_aces: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            max_bet: None,
            min_bet: Some(100),
            blackjack_payout: BlackjackPayout::ThreeToTwo,
            dealer_soft_17: DealerSoft17Action::Stand,
            insurance: false,
            early_surrender: false,
            late_surrender: true,
            max_splits: Some(5),
            double_after_split: true,
            split_aces: true,
        }
    }
}

