#![allow(clippy::result_large_err)]

use std::fmt;

use crate::card::hand::{DealerHand, PlayerHand, PlayerTurn, Status};
use crate::card::shoe::Shoe;
use crate::rules::Rules;
use crate::state::GameState;
use crate::statistics::Statistics;

/// The player's options for playing their hand
#[derive(Debug, PartialEq, Eq)]
pub enum HandAction {
    Stand,
    Hit,
    Double,
    Split,
    Surrender,
}

/// The game input. Different states require different inputs.
#[derive(Debug)]
pub enum Input {
    Bet(u32),
    Choice(bool),
    Action(HandAction),
}

/// The game table. This is where the game is played.
/// It holds the player's chips, the shoe, and the game rules.
#[derive(Debug)]
pub struct Table {
    pub chips: u32,
    pub shoe: Shoe,
    pub rules: Rules,
    pub statistics: Statistics,
    is_simulation: bool, // Only stop at the states requiring user input and do not validate the input
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransitionError {
    WrongInput,
    BetError(BetError),
    DoubleError(DoubleError),
    SplitError(SplitError),
    SurrenderError(SurrenderError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BetError {
    BetTooLow,
    BetTooHigh,
    CantAfford,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DoubleError {
    CantAfford,
    NotTwoCards,
    DoubleAfterSplitNotAllowed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SplitError {
    CantAfford,
    NotPair,
    MaxSplitsReached,
    SplitAcesNotAllowed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SurrenderError {
    NotTwoCards,
    LateSurrenderNotAllowed,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongInput => write!(f, "Wrong input"),
            Self::BetError(err) => match err {
                BetError::BetTooLow => write!(f, "Bet too low"),
                BetError::BetTooHigh => write!(f, "Bet too high"),
                BetError::CantAfford => write!(f, "Can't afford bet"),
            },
            Self::DoubleError(err) => match err {
                DoubleError::CantAfford => write!(f, "Can't afford double down"),
                DoubleError::NotTwoCards => write!(f, "Not two cards"),
                DoubleError::DoubleAfterSplitNotAllowed => {
                    write!(f, "Double after split not allowed")
                }
            },
            Self::SplitError(err) => match err {
                SplitError::CantAfford => write!(f, "Can't afford split"),
                SplitError::NotPair => write!(f, "Not a pair"),
                SplitError::MaxSplitsReached => write!(f, "Max splits reached"),
                SplitError::SplitAcesNotAllowed => write!(f, "Split aces not allowed"),
            },
            Self::SurrenderError(err) => match err {
                SurrenderError::NotTwoCards => write!(f, "Not two cards"),
                SurrenderError::LateSurrenderNotAllowed => write!(f, "Late surrender not allowed"),
            },
        }
    }
}

/// If the player input is invalid, the game cannot progress.
/// In these cases, the game returns an error with the unchanged state.
pub type FallibleTransition = Result<GameState, (GameState, TransitionError)>;

impl Table {
    #[must_use]
    pub const fn new(chips: u32, shoe: Shoe, rules: Rules) -> Self {
        Self {
            chips,
            shoe,
            rules,
            statistics: Statistics::new(),
            is_simulation: false,
        }
    }

    /// Plays the game from the given state and input.
    /// Returns the next state of the game, or the same state if the game could not progress.
    /// # Errors
    /// Returns Err with the same state if the game could not progress.
    pub fn play(&mut self, state: GameState, input: Option<Input>) -> FallibleTransition {
        match (state, input) {
            (GameState::Betting, input) => {
                if let Some(Input::Bet(bet)) = input {
                    self.bet(bet)
                } else {
                    Err((GameState::Betting, TransitionError::WrongInput))
                }
            }
            (GameState::DealFirstPlayerCard(bet), _) => Ok(self.deal_first_player_card(bet)),
            (GameState::DealFirstDealerCard(player_hand), _) => {
                Ok(self.deal_first_dealer_card(player_hand))
            }
            (GameState::DealSecondPlayerCard(player_hand, dealer_hand), _) => {
                Ok(self.deal_second_player_card(player_hand, dealer_hand))
            }
            (GameState::DealHoleCard(player_hand, dealer_hand), _) => {
                Ok(self.deal_hole_card(player_hand, dealer_hand))
            }
            (GameState::OfferEarlySurrender(player_hand, dealer_hand), input) => {
                if let Some(Input::Choice(early_surrender)) = input {
                    Ok(self.choose_early_surrender(player_hand, dealer_hand, early_surrender))
                } else {
                    Err((
                        GameState::OfferEarlySurrender(player_hand, dealer_hand),
                        TransitionError::WrongInput,
                    ))
                }
            }
            (GameState::OfferInsurance(player_hand, dealer_hand), input) => {
                if let Some(Input::Bet(insurance_bet)) = input {
                    self.bet_insurance(player_hand, dealer_hand, insurance_bet)
                } else {
                    Err((
                        GameState::OfferInsurance(player_hand, dealer_hand),
                        TransitionError::WrongInput,
                    ))
                }
            }
            (GameState::CheckDealerHoleCard(player_hand, dealer_hand, insurance_bet), _) => {
                Ok(self.check_dealer_hole_card(player_hand, dealer_hand, insurance_bet))
            }
            (GameState::PlayPlayerTurn(turn, dealer_hand, insurance_bet), input) => {
                if let Some(Input::Action(action)) = input {
                    self.play_player_turn(turn, dealer_hand, insurance_bet, action)
                } else {
                    Err((
                        GameState::PlayPlayerTurn(turn, dealer_hand, insurance_bet),
                        TransitionError::WrongInput,
                    ))
                }
            }
            (GameState::Stand(turn, dealer_hand, insurance_bet), _) => {
                Ok(self.stand(turn, dealer_hand, insurance_bet))
            }
            (GameState::Hit(turn, dealer_hand, insurance_bet), _) => {
                Ok(self.hit(turn, dealer_hand, insurance_bet))
            }
            (GameState::Double(turn, dealer_hand, insurance_bet), _) => {
                Ok(self.double(turn, dealer_hand, insurance_bet))
            }
            (GameState::Split(turn, dealer_hand, insurance_bet), _) => {
                Ok(self.split(turn, dealer_hand, insurance_bet))
            }
            (GameState::DealFirstSplitCard(turn, new_hand, dealer_hand, insurance_bet), _) => {
                Ok(self.deal_first_split_card(turn, new_hand, dealer_hand, insurance_bet))
            }
            (GameState::DealSecondSplitCard(turn, new_hand, dealer_hand, insurance_bet), _) => {
                Ok(self.deal_second_split_card(turn, new_hand, dealer_hand, insurance_bet))
            }
            (GameState::Surrender(turn, dealer_hand, insurance_bet), _) => {
                Ok(self.late_surrender(turn, dealer_hand, insurance_bet))
            }
            (GameState::RevealHoleCard(player_hands, dealer_hand, insurance_bet), _) => {
                Ok(self.play_dealer_turn_or_end_round(player_hands, dealer_hand, insurance_bet))
            }
            (GameState::PlayDealerTurn(player_hands, dealer_hand, insurance_bet), _) => {
                Ok(self.play_dealer_turn(player_hands, dealer_hand, insurance_bet))
            }
            (GameState::RoundOver(player_hands, dealer_hand, insurance_bet), _) => {
                Ok(self.end_round(player_hands, dealer_hand, insurance_bet))
            }
            (GameState::Payout(_, winnings), _) => Ok(self.pay_out_winnings(winnings)),
            (GameState::Shuffle, _) => Ok(self.shuffle_dispenser()),
            (GameState::GameOver, _) => Err((GameState::GameOver, TransitionError::WrongInput)),
        }
    }

    /// A helper function to determine if the player is allowed to double down on their current hand.
    /// The player can double down if their hand consists of two cards, they have enough chips,
    /// and the maximum splits and double after split rules do not prevent them from doing so.
    /// # Errors
    /// Returns an error containing the reason why the player cannot double down.
    pub fn check_double_allowed(&self, hand: &PlayerHand) -> Result<(), DoubleError> {
        if hand.size() != 2 {
            Err(DoubleError::NotTwoCards)
        } else if hand.bet > self.chips {
            Err(DoubleError::CantAfford)
        } else if hand.splits > 0 && !self.rules.double_after_split {
            Err(DoubleError::DoubleAfterSplitNotAllowed)
        } else {
            Ok(())
        }
    }

    /// A helper function to determine if the player is allowed to split their current hand.
    /// The player can split if their hand is a pair, they have enough chips to double their bet,
    /// and the maximum splits and split-aces rules do not prevent them from doing so.
    /// # Errors
    /// Returns an error containing the reason why the player cannot split.
    pub fn check_split_allowed(&self, hand: &PlayerHand) -> Result<(), SplitError> {
        if !hand.is_pair() {
            Err(SplitError::NotPair)
        } else if hand.bet > self.chips {
            Err(SplitError::CantAfford)
        } else if self
            .rules
            .max_splits
            .map_or(false, |max| hand.splits >= max)
        {
            Err(SplitError::MaxSplitsReached)
        } else if hand.value.soft && !self.rules.split_aces {
            Err(SplitError::SplitAcesNotAllowed)
        } else {
            Ok(())
        }
    }

    /// A helper function to determine if the player is allowed to surrender their current hand.
    /// The player can surrender if their hand consists of two cards and late surrendering
    /// is enabled in the game configuration.
    /// # Errors
    /// Returns an error containing the reason why the player cannot surrender.
    pub fn check_surrender_allowed(&self, hand: &PlayerHand) -> Result<(), SurrenderError> {
        if hand.size() != 2 {
            Err(SurrenderError::NotTwoCards)
        } else if !self.rules.offer_late_surrender {
            Err(SurrenderError::LateSurrenderNotAllowed)
        } else {
            Ok(())
        }
    }

    /// The player places a bet to start the round.
    /// The bet must be within the table limits and the player must have enough chips.
    /// If the bet is valid, the game transitions to dealing the first player card.
    fn bet(&mut self, bet: u32) -> FallibleTransition {
        if self.is_simulation {
            self.chips -= bet;
            // Simulated bets should already be valid, so we don't need to check them
            Ok(self.deal_first_player_card(bet))
        } else {
            match (self.rules.min_bet, self.rules.max_bet) {
                (Some(min), _) if bet < min => Err((
                    GameState::Betting,
                    TransitionError::BetError(BetError::BetTooLow),
                )),
                (_, Some(max)) if bet > max => Err((
                    GameState::Betting,
                    TransitionError::BetError(BetError::BetTooHigh),
                )),
                _ if bet > self.chips => Err((
                    GameState::Betting,
                    TransitionError::BetError(BetError::CantAfford),
                )),
                _ => {
                    self.chips -= bet;
                    Ok(GameState::DealFirstPlayerCard(bet))
                }
            }
        }
    }

    /// The dealer deals the first card to the player and the player's hand is created.
    /// Next, the dealer will deal their first card.
    fn deal_first_player_card(&mut self, bet: u32) -> GameState {
        let card = self.shoe.draw_card();
        let player_hand = PlayerHand::new(card, bet);
        if self.is_simulation {
            self.deal_first_dealer_card(player_hand)
        } else {
            GameState::DealFirstDealerCard(player_hand)
        }
    }

    /// The dealer deals the first card to themselves and the dealer's hand is created.
    /// Next, the dealer will deal the second card to the player.
    fn deal_first_dealer_card(&mut self, player_hand: PlayerHand) -> GameState {
        let card = self.shoe.draw_card();
        let dealer_hand = DealerHand::new(card, self.rules.dealer_soft_17);
        if self.is_simulation {
            self.deal_second_player_card(player_hand, dealer_hand)
        } else {
            GameState::DealSecondPlayerCard(player_hand, dealer_hand)
        }
    }

    /// The dealer deals the second card to the player.
    /// Next, the dealer will deal the second card to themselves, also known as the hole card.
    fn deal_second_player_card(
        &mut self,
        mut player_hand: PlayerHand,
        dealer_hand: DealerHand,
    ) -> GameState {
        player_hand += self.shoe.draw_card();
        if self.is_simulation {
            self.deal_hole_card(player_hand, dealer_hand)
        } else {
            GameState::DealHoleCard(player_hand, dealer_hand)
        }
    }

    /// The dealer deals the hole card to themselves.
    /// If the dealer is showing a 10 or an Ace, they will check their hole card for Blackjack.
    /// If early surrender or insurance is offered, the game will transition to those states first.
    /// If the dealer cannot have Blackjack or if the player does have Blackjack, the dealer will
    /// not check their hole card, and will instead let the player play their hand.
    fn deal_hole_card(
        &mut self,
        player_hand: PlayerHand,
        mut dealer_hand: DealerHand,
    ) -> GameState {
        dealer_hand += self.shoe.draw_card();
        if dealer_hand.showing() < 10 || player_hand.status == Status::Blackjack {
            let player_turn = PlayerTurn::new(player_hand);
            self.play_player_turn_or_go_to_dealer_turn(player_turn, dealer_hand, 0)
        } else if self.rules.offer_early_surrender {
            GameState::OfferEarlySurrender(player_hand, dealer_hand)
        } else if self.rules.offer_insurance && dealer_hand.showing() == 11 {
            GameState::OfferInsurance(player_hand, dealer_hand)
        } else if self.is_simulation {
            self.check_dealer_hole_card(player_hand, dealer_hand, 0)
        } else {
            GameState::CheckDealerHoleCard(player_hand, dealer_hand, 0)
        }
    }

    /// The player decides whether to surrender early.
    /// If the player surrenders, their hand is finished and the round is over.
    /// Otherwise, if insurance is offered and the dealer is showing an Ace, the player can place
    /// an insurance bet.
    /// Otherwise, the dealer checks their hole card for Blackjack.
    fn choose_early_surrender(
        &mut self,
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
        surrender: bool,
    ) -> GameState {
        match (surrender, self.is_simulation) {
            (true, true) => self.late_surrender(PlayerTurn::new(player_hand), dealer_hand, 0),
            (true, false) => GameState::Surrender(PlayerTurn::new(player_hand), dealer_hand, 0),
            (false, _) if self.rules.offer_insurance && dealer_hand.showing() == 11 => {
                GameState::OfferInsurance(player_hand, dealer_hand)
            }
            (false, true) => self.check_dealer_hole_card(player_hand, dealer_hand, 0),
            (false, false) => GameState::CheckDealerHoleCard(player_hand, dealer_hand, 0),
        }
    }

    /// The player places an insurance bet.
    /// The bet must be less than half of the player's original bet,
    /// and the player must have enough chips.
    /// Next, the dealer will check their hole card for Blackjack.
    fn bet_insurance(
        &mut self,
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> FallibleTransition {
        if self.is_simulation {
            // Simulated bets should already be valid, so we don't need to check them
            self.chips -= insurance_bet;
            Ok(self.check_dealer_hole_card(player_hand, dealer_hand, insurance_bet))
        } else if insurance_bet > player_hand.bet / 2 {
            Err((
                GameState::OfferInsurance(player_hand, dealer_hand),
                TransitionError::BetError(BetError::BetTooHigh),
            ))
        } else if insurance_bet > self.chips {
            Err((
                GameState::OfferInsurance(player_hand, dealer_hand),
                TransitionError::BetError(BetError::CantAfford),
            ))
        } else {
            self.chips -= insurance_bet;
            Ok(GameState::CheckDealerHoleCard(
                player_hand,
                dealer_hand,
                insurance_bet,
            ))
        }
    }

    /// The dealer checks their hole card for Blackjack.
    /// If the dealer does not have Blackjack, it is the player's turn to play their hand.
    /// If the dealer does have Blackjack, the round is over.
    fn check_dealer_hole_card(
        &mut self,
        player_hand: PlayerHand,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        if dealer_hand.status == Status::Blackjack {
            if self.is_simulation {
                self.end_round(vec![player_hand], dealer_hand, insurance_bet)
            } else {
                GameState::RoundOver(vec![player_hand], dealer_hand, insurance_bet)
            }
        } else {
            self.play_player_turn_or_go_to_dealer_turn(
                PlayerTurn::new(player_hand),
                dealer_hand,
                insurance_bet,
            )
        }
    }

    /// It is the player's turn.
    /// The player starts with one hand, but may have more if they split.
    /// The player can choose to stand, hit, double down, split, or surrender.
    /// Doubling down, splitting, and surrendering are only allowed in certain circumstances.
    fn play_player_turn(
        &mut self,
        turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
        action: HandAction,
    ) -> FallibleTransition {
        match action {
            HandAction::Hit if self.is_simulation => Ok(self.hit(turn, dealer_hand, insurance_bet)),
            HandAction::Hit => Ok(GameState::Hit(turn, dealer_hand, insurance_bet)),
            HandAction::Stand if self.is_simulation => {
                Ok(self.stand(turn, dealer_hand, insurance_bet))
            }
            HandAction::Stand => Ok(GameState::Stand(turn, dealer_hand, insurance_bet)),
            // Simulated moves should already be valid, so we don't need to check them
            HandAction::Double if self.is_simulation => {
                self.chips -= turn.current_hand.bet;
                Ok(self.double(turn, dealer_hand, insurance_bet))
            }
            HandAction::Double => {
                if let Err(err) = self.check_double_allowed(&turn.current_hand) {
                    Err((
                        GameState::PlayPlayerTurn(turn, dealer_hand, insurance_bet),
                        TransitionError::DoubleError(err),
                    ))
                } else {
                    self.chips -= turn.current_hand.bet;
                    Ok(GameState::Double(turn, dealer_hand, insurance_bet))
                }
            }
            // Simulated moves should already be valid, so we don't need to check them
            HandAction::Split if self.is_simulation => {
                self.chips -= turn.current_hand.bet;
                Ok(self.split(turn, dealer_hand, insurance_bet))
            }
            HandAction::Split => {
                if let Err(err) = self.check_split_allowed(&turn.current_hand) {
                    Err((
                        GameState::PlayPlayerTurn(turn, dealer_hand, insurance_bet),
                        TransitionError::SplitError(err),
                    ))
                } else {
                    self.chips -= turn.current_hand.bet;
                    Ok(GameState::Split(turn, dealer_hand, insurance_bet))
                }
            }
            // Simulated moves should already be valid, so we don't need to check them
            HandAction::Surrender if self.is_simulation => {
                Ok(self.late_surrender(turn, dealer_hand, insurance_bet))
            }
            HandAction::Surrender => {
                if let Err(err) = self.check_surrender_allowed(&turn.current_hand) {
                    Err((
                        GameState::PlayPlayerTurn(turn, dealer_hand, insurance_bet),
                        TransitionError::SurrenderError(err),
                    ))
                } else {
                    Ok(GameState::Surrender(turn, dealer_hand, insurance_bet))
                }
            }
        }
    }

    /// The dealer deals the next card to the player's current hand.
    /// We continue the player's turn if they still have hands in play, or go to the dealer's turn.
    fn hit(
        &mut self,
        mut turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        turn.current_hand += self.shoe.draw_card();
        self.play_player_turn_or_go_to_dealer_turn(turn, dealer_hand, insurance_bet)
    }

    /// The player stands and the hand is finished.
    /// We continue the player's turn if they still have hands in play, or go to the dealer's turn.
    fn stand(
        &mut self,
        mut turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        turn.current_hand.stand();
        self.play_player_turn_or_go_to_dealer_turn(turn, dealer_hand, insurance_bet)
    }

    /// The player doubles down and the hand is finished.
    /// We continue the player's turn if they still have hands in play, or go to the dealer's turn.
    fn double(
        &mut self,
        mut turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        turn.current_hand.double(self.shoe.draw_card());
        self.play_player_turn_or_go_to_dealer_turn(turn, dealer_hand, insurance_bet)
    }

    /// The dealer separates the player's hand into two.
    /// Next, the dealer will deal a new card to the first of the two split hands.
    fn split(
        &mut self,
        mut turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        let new_hand = turn.current_hand.split();
        if self.is_simulation {
            self.deal_first_split_card(turn, new_hand, dealer_hand, insurance_bet)
        } else {
            GameState::DealFirstSplitCard(turn, new_hand, dealer_hand, insurance_bet)
        }
    }

    /// The dealer deals a card to the first of the two split hands.
    /// Next, the dealer will deal a new card to the second of the two split hands.
    fn deal_first_split_card(
        &mut self,
        mut turn: PlayerTurn,
        new_hand: PlayerHand,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        turn.current_hand += self.shoe.draw_card();
        if self.is_simulation {
            self.deal_second_split_card(turn, new_hand, dealer_hand, insurance_bet)
        } else {
            GameState::DealSecondSplitCard(turn, new_hand, dealer_hand, insurance_bet)
        }
    }

    /// The dealer deals a card to the second of the two split hands.
    /// The player will play the first split hand first, and the second split hand after.
    /// We continue the player's turn if they still have hands in play, or go to the dealer's turn.
    fn deal_second_split_card(
        &mut self,
        mut turn: PlayerTurn,
        mut new_hand: PlayerHand,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        new_hand += self.shoe.draw_card();
        turn.defer(new_hand);
        self.play_player_turn_or_go_to_dealer_turn(turn, dealer_hand, insurance_bet)
    }

    /// The player surrenders and the hand is finished.
    /// We continue the player's turn if they still have hands in play, or go to the dealer's turn.
    fn late_surrender(
        &mut self,
        mut turn: PlayerTurn,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        turn.current_hand.surrender();
        self.play_player_turn_or_go_to_dealer_turn(turn, dealer_hand, insurance_bet)
    }

    /// We check if the player still has a hand to play.
    /// If so, we continue the player's turn.
    /// Otherwise, the player's turn is over and the dealer will reveal their hole card.
    fn play_player_turn_or_go_to_dealer_turn(
        &mut self,
        player_turn: PlayerTurn,
        mut dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        match player_turn.continue_playing() {
            Ok(turn) => GameState::PlayPlayerTurn(turn, dealer_hand, insurance_bet),
            Err(player_hands) => {
                // If the player did not explicitly stand on any of their hands,
                // the dealer will simply flip their hole card and stand immediately.
                if dealer_hand.status == Status::InPlay
                    && !player_hands.iter().any(|hand| hand.status == Status::Stood)
                {
                    dealer_hand.status = Status::Stood;
                }
                if self.is_simulation {
                    self.play_dealer_turn_or_end_round(player_hands, dealer_hand, insurance_bet)
                } else {
                    GameState::RevealHoleCard(player_hands, dealer_hand, insurance_bet)
                }
            }
        }
    }

    /// The dealer reveals their hole card.
    /// If the dealer's hand is no longer in play, the round is over.
    fn play_dealer_turn_or_end_round(
        &mut self,
        player_hands: Vec<PlayerHand>,
        dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        match (self.is_simulation, dealer_hand.status == Status::InPlay) {
            (true, true) => self.play_dealer_turn(player_hands, dealer_hand, insurance_bet),
            (false, true) => GameState::PlayDealerTurn(player_hands, dealer_hand, insurance_bet),
            (true, false) => self.end_round(player_hands, dealer_hand, insurance_bet),
            (false, false) => GameState::RoundOver(player_hands, dealer_hand, insurance_bet),
        }
    }

    /// The dealer adds a card to their hand.
    /// This will repeat itself until the dealer's hand is no longer in play.
    /// Then, the round is over.
    fn play_dealer_turn(
        &mut self,
        player_hands: Vec<PlayerHand>,
        mut dealer_hand: DealerHand,
        insurance_bet: u32,
    ) -> GameState {
        dealer_hand += self.shoe.draw_card();
        self.play_dealer_turn_or_end_round(player_hands, dealer_hand, insurance_bet)
    }

    /// The round is over.
    /// The player's total bet and winnings are calculated.
    fn end_round(
        &mut self,
        mut player_hands: Vec<PlayerHand>,
        dealer_hand: DealerHand,
        insurance: u32,
    ) -> GameState {
        let total_bet = player_hands.iter().map(|hand| hand.bet).sum::<u32>() + insurance;
        for hand in &mut player_hands {
            hand.winnings = hand.calculate_winnings(&dealer_hand, self.rules.blackjack_payout);
        }
        let mut winnings = player_hands.iter().map(|hand| hand.winnings).sum();
        if dealer_hand.status == Status::Blackjack {
            winnings += insurance * 2;
        }
        self.statistics.update(player_hands, dealer_hand);
        if self.is_simulation {
            self.pay_out_winnings(winnings)
        } else {
            GameState::Payout(total_bet, winnings)
        }
    }

    /// The dealer pays out the player's winnings.
    /// If the player has no chips left, the game is over.
    /// Otherwise, the dealer will shuffle the shoe if necessary, or the game will return to betting.
    fn pay_out_winnings(&mut self, winnings: u32) -> GameState {
        self.chips += winnings;
        if self.rules.min_bet.map_or(self.chips == 0, |min| self.chips < min) {
            return GameState::GameOver;
        }
        match (self.is_simulation, self.shoe.needs_shuffle()) {
            (true, true) => self.shuffle_dispenser(),
            (false, true) => GameState::Shuffle,
            (_, false) => GameState::Betting,
        }
    }

    /// The dealer shuffles the shoe.
    /// The game returns to the betting state.
    fn shuffle_dispenser(&mut self) -> GameState {
        self.shoe.shuffle();
        GameState::Betting
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bet() {
        let mut table = Table::new(
            100,
            Shoe::new(4, 0.50),
            Rules {
                min_bet: Some(1),
                max_bet: Some(100),
                ..Rules::default()
            },
        );
        assert_eq!(table.bet(50), Ok(GameState::DealFirstPlayerCard(50)));
        assert_eq!(
            table.bet(101),
            Err((
                GameState::Betting,
                TransitionError::BetError(BetError::BetTooHigh)
            ))
        );
        assert_eq!(
            table.bet(0),
            Err((
                GameState::Betting,
                TransitionError::BetError(BetError::BetTooLow)
            ))
        );
        assert_eq!(table.bet(1), Ok(GameState::DealFirstPlayerCard(1)));
        assert_eq!(
            table.bet(50),
            Err((
                GameState::Betting,
                TransitionError::BetError(BetError::CantAfford)
            ))
        );
    }
}
