#![allow(clippy::match_same_arms, clippy::unnested_or_patterns)]

//! Basic strategy for playing blackjack.
//! This simulates a player who knows the optimal move for every possible hand.
//! This makes a best-effort attempt to consider the rules of the game, but is not perfect.

use crate::card::hand::{DealerHand, PlayerHand, PlayerTurn};
use crate::composed;
use crate::game::{HandAction, Table};

#[must_use]
pub const fn bet() -> u32 {
    100 // TODO: Factor out betting strategy
}

#[must_use]
pub fn surrender_late(table: &Table, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> bool {
    match (player_hand.value.total, dealer_hand.showing()) {
        (14, 10) => table.shoe.decks == 1 && player_hand.is_pair(),
        (14, 11) => table.shoe.decks == 1 && player_hand.is_pair() && dealer_hand.hits_on_soft_17(),
        (15, 10) if table.shoe.decks < 8 => composed!(player_hand => 9, 6; 10, 5),
        (15, 10) if table.shoe.decks >= 8 => true,
        (15, 11) if table.shoe.decks < 4 => {
            dealer_hand.hits_on_soft_17() && composed!(player_hand => 9, 6; 10, 5)
        }
        (15, 11) if table.shoe.decks >= 4 => true,
        (16, 9) => table.shoe.decks >= 4,
        (16, 10) => true,
        (16, 11) if table.shoe.decks == 1 && !dealer_hand.hits_on_soft_17() => {
            composed!(player_hand => 10, 6)
        }
        (16, 11) if table.shoe.decks <= 2 && dealer_hand.hits_on_soft_17() => {
            composed!(player_hand => 9, 7; 10, 6)
        }
        (16, 11) if table.shoe.decks == 2 && !dealer_hand.hits_on_soft_17() => true,
        (16, 11) if table.shoe.decks > 2 => true,
        (15 | 17, 11) => dealer_hand.hits_on_soft_17(),
        _ => false,
    }
}

#[must_use]
pub fn surrender_early(table: &Table, player_hand: &PlayerHand, dealer_hand: &DealerHand) -> bool {
    match (player_hand.value.soft, player_hand.is_pair()) {
        (false, false) => surrender_early_hard(player_hand, dealer_hand),
        (true, false) => false, // Soft (non-pair) hands should not be surrendered early
        (_, true) => surrender_early_pair(player_hand, dealer_hand, table),
    }
}

/// Source: <https://wizardofodds.com/games/blackjack/surrender/>
fn surrender_early_hard(player_hand: &PlayerHand, dealer_hand: &DealerHand) -> bool {
    match (player_hand.value.total, dealer_hand.showing()) {
        (5..=7, 11) => true,
        (12..=17, 11) => true,
        (14..=16, 10) => true,
        _ => false,
    }
}

/// Source: <https://wizardofodds.com/games/blackjack/surrender/>
fn surrender_early_pair(player_hand: &PlayerHand, dealer_hand: &DealerHand, table: &Table) -> bool {
    match (player_hand.value.total / 2, dealer_hand.showing()) {
        (8, 10) if table.shoe.decks == 1 && table.rules.double_after_split => false,
        (7..=8, 10) | (3 | 6..=8, 11) => true,
        (2, 11) if dealer_hand.hits_on_soft_17() => true,
        _ => false,
    }
}

#[must_use]
pub const fn bet_insurance() -> u32 {
    0
}

/// The preferred action which may involve a fallback action
enum PreferredAction {
    Stand,
    Hit,
    Split,
    DoubleOrHit,
    DoubleOrStand,
    SurrenderOrHit,
    SurrenderOrStand,
    SurrenderOrSplit,
    SplitIfDoubleAfterSplitAllowedElseHit,
}

/// Assuming 4-8 decks
#[must_use]
pub fn play_hand(table: &Table, player_hands: &PlayerTurn, dealer_hand: &DealerHand) -> HandAction {
    let preferred = match (
        player_hands.current_hand().value.soft,
        table.check_split_allowed(player_hands).is_ok(),
    ) {
        (false, false) => make_move_hard(table, &player_hands.current_hand(), dealer_hand),
        (true, false) => make_move_soft(&player_hands.current_hand(), dealer_hand),
        (_, true) => make_move_splittable(&player_hands.current_hand(), dealer_hand),
    };
    match preferred {
        PreferredAction::Stand => HandAction::Stand,
        PreferredAction::Hit => HandAction::Hit,
        PreferredAction::Split => HandAction::Split,
        PreferredAction::DoubleOrHit => {
            if table.check_double_allowed(player_hands).is_ok() {
                HandAction::Double
            } else {
                HandAction::Hit
            }
        }
        PreferredAction::DoubleOrStand => {
            if table.check_double_allowed(player_hands).is_ok() {
                HandAction::Double
            } else {
                HandAction::Stand
            }
        }
        PreferredAction::SurrenderOrHit => {
            if table
                .check_surrender_allowed(&player_hands.current_hand())
                .is_ok()
            {
                HandAction::Surrender
            } else {
                HandAction::Hit
            }
        }
        PreferredAction::SurrenderOrStand => {
            if table
                .check_surrender_allowed(&player_hands.current_hand())
                .is_ok()
            {
                HandAction::Surrender
            } else {
                HandAction::Stand
            }
        }
        PreferredAction::SurrenderOrSplit => {
            if table
                .check_surrender_allowed(&player_hands.current_hand())
                .is_ok()
            {
                HandAction::Surrender
            } else {
                HandAction::Split
            }
        }
        PreferredAction::SplitIfDoubleAfterSplitAllowedElseHit => {
            if table.rules.double_after_split {
                HandAction::Split
            } else {
                HandAction::Hit
            }
        }
    }
}

fn make_move_hard(
    table: &Table,
    player_hand: &PlayerHand,
    dealer_hand: &DealerHand,
) -> PreferredAction {
    match (player_hand.value.total, dealer_hand.showing()) {
        (4..=8, 2..=11) => PreferredAction::Hit,
        (9, 2) => {
            if table.shoe.decks <= 2 {
                PreferredAction::DoubleOrHit
            } else {
                PreferredAction::Hit
            }
        }
        (9, 3..=6) => PreferredAction::DoubleOrHit,
        (9, 7..=11) => PreferredAction::Hit,
        (10, 2..=9) => PreferredAction::DoubleOrHit,
        (10, 10 | 11) => PreferredAction::Hit,
        (11, 2..=10) => PreferredAction::DoubleOrHit,
        (11, 11) => {
            if table.shoe.decks <= 2 || dealer_hand.hits_on_soft_17() {
                PreferredAction::DoubleOrHit
            } else {
                PreferredAction::Hit
            }
        }
        (12, 2..=3) => PreferredAction::Hit,
        (12, 4..=6) => PreferredAction::Stand,
        (12..=14, 7..=11) => PreferredAction::Hit,
        (13..=16, 2..=6) => PreferredAction::Stand,
        (15, 7..=9) => PreferredAction::Hit,
        (15, 10) => PreferredAction::SurrenderOrHit,
        (15, 11) => {
            if dealer_hand.hits_on_soft_17() {
                PreferredAction::SurrenderOrHit
            } else {
                PreferredAction::Hit
            }
        }
        (16, 7 | 8) => PreferredAction::Hit,
        (16, 9..=11) => PreferredAction::SurrenderOrHit,
        (17, 2..=10) => PreferredAction::Stand,
        (17, 11) => {
            if dealer_hand.hits_on_soft_17() {
                PreferredAction::SurrenderOrStand
            } else {
                PreferredAction::Stand
            }
        }
        (18..=21, 2..=11) => PreferredAction::Stand,
        (_, showing) => panic!(
            "Invalid hand value: {} against {}",
            player_hand.value, showing
        ),
    }
}

fn make_move_soft(player_hand: &PlayerHand, dealer_hand: &DealerHand) -> PreferredAction {
    match (player_hand.value.total, dealer_hand.showing()) {
        (13 | 14, 2..=4) => PreferredAction::Hit,
        (13 | 14, 5 | 6) => PreferredAction::DoubleOrHit,
        (15 | 16, 2 | 3) => PreferredAction::Hit,
        (15 | 16, 4..=6) => PreferredAction::DoubleOrHit,
        (17, 2) => PreferredAction::Hit,
        (17, 3..=6) => PreferredAction::DoubleOrHit,
        (13..=17, 7..=11) => PreferredAction::Hit,
        (18, 2) => {
            if dealer_hand.hits_on_soft_17() {
                PreferredAction::DoubleOrStand
            } else {
                PreferredAction::Stand
            }
        }
        (18, 3..=6) => PreferredAction::DoubleOrStand,
        (18, 7 | 8) => PreferredAction::Stand,
        (18, 9..=11) => PreferredAction::Hit,
        (19, 2..=5) => PreferredAction::Stand,
        (19, 6) => {
            if dealer_hand.hits_on_soft_17() {
                PreferredAction::DoubleOrStand
            } else {
                PreferredAction::Stand
            }
        }
        (19, 7..=11) => PreferredAction::Stand,
        (20 | 21, 2..=11) => PreferredAction::Stand,
        (_, showing) => panic!(
            "Invalid hand value: {} against {}",
            player_hand.value, showing
        ),
    }
}

fn make_move_splittable(player_hand: &PlayerHand, dealer_hand: &DealerHand) -> PreferredAction {
    match (player_hand.value.total / 2, dealer_hand.showing()) {
        (2 | 3, 2 | 3) => PreferredAction::SplitIfDoubleAfterSplitAllowedElseHit,
        (2 | 3, 4..=7) => PreferredAction::Split,
        (2 | 3, 8..=11) => PreferredAction::Hit,
        (4, 2..=4) => PreferredAction::Hit,
        (4, 5..=6) => PreferredAction::SplitIfDoubleAfterSplitAllowedElseHit,
        (4, 7..=11) => PreferredAction::Hit,
        (5, 2..=9) => PreferredAction::DoubleOrHit,
        (5, 10 | 11) => PreferredAction::Hit,
        (6, 2) => PreferredAction::SplitIfDoubleAfterSplitAllowedElseHit,
        (6, 3..=6) => PreferredAction::Split,
        (6, 7..=11) => PreferredAction::Hit,
        (7, 2..=7) => PreferredAction::Split,
        (7, 8..=11) => PreferredAction::Hit,
        (8, 2..=10) => PreferredAction::Split,
        (8, 11) => {
            if dealer_hand.hits_on_soft_17() {
                PreferredAction::SurrenderOrSplit
            } else {
                PreferredAction::Split
            }
        }
        (9, 2..=6) => PreferredAction::Split,
        (9, 7) => PreferredAction::Stand,
        (9, 8 | 9) => PreferredAction::Split,
        (9, 10 | 11) => PreferredAction::Stand,
        (10, 2..=11) => PreferredAction::Stand,
        (11, 2..=11) => PreferredAction::Split,
        (_, showing) => panic!(
            "Invalid hand value: {} against {}",
            player_hand.value, showing
        ),
    }
}
