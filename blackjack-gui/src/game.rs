use crate::input::InputField;

use blackjack_core::basic_strategy;
use blackjack_core::card::shoe::Shoe;
use blackjack_core::game::{Error, Input, Table};
use blackjack_core::rules::Rules;
use blackjack_core::state::GameState;

use crossterm::event::KeyCode;

#[derive(Debug)]
pub struct Blackjack {
    pub table: Table,
    pub game_state: GameState,
    pub input_field: Option<InputField>,
    pub last_error: Option<Error>,
}

impl Default for Blackjack {
    fn default() -> Self {
        Self::new()
    }
}
impl Blackjack {
    pub fn new() -> Self {
        let table = Table::new(50000, Shoe::new(4, 0.50), Rules::default());
        let game_state = GameState::Betting;
        let input_field = InputField::from_game(&game_state, &table);
        Self {
            table,
            game_state,
            input_field,
            last_error: None,
        }
    }

    pub fn tick(&mut self) {
        if self.try_progress(None).is_ok() {
            self.last_error = None;
        }
    }

    pub fn input(&mut self, key: KeyCode) {
        let input = self.input_field.as_mut().and_then(|f| f.consider(key));
        if input.is_some() {
            if let Err(transition_error) = self.try_progress(input) {
                self.last_error = Some(transition_error);
            } else {
                self.last_error = None;
            }
        }
    }

    pub fn simulate(&mut self) {
        let input = self.basic_strategy_input();
        if let Err(transition_error) = self.try_progress(input) {
            self.last_error = Some(transition_error);
        } else {
            self.last_error = None;
        }
    }

    fn try_progress(&mut self, input: Option<Input>) -> Result<(), Error> {
        let current_state = core::mem::replace(&mut self.game_state, GameState::Betting);
        match self.table.progress(current_state, input) {
            Ok(next_state) => {
                self.input_field = InputField::from_game(&next_state, &self.table);
                self.game_state = next_state;
                Ok(())
            }
            Err((same_state, transition_error)) => {
                self.game_state = same_state;
                Err(transition_error)
            }
        }
    }

    pub fn basic_strategy_input(&self) -> Option<Input> {
        match &self.game_state {
            GameState::Betting => Some(Input::Bet(basic_strategy::bet())),
            GameState::OfferEarlySurrender {
                player_hand,
                dealer_hand,
            } => Some(Input::Choice(basic_strategy::surrender_early(
                &self.table,
                player_hand,
                dealer_hand,
            ))),
            GameState::OfferInsurance { .. } => Some(Input::Bet(basic_strategy::bet_insurance())),
            GameState::PlayPlayerTurn {
                player_turn,
                dealer_hand,
                ..
            } => Some(Input::Action(basic_strategy::play_hand(
                &self.table,
                player_turn,
                dealer_hand,
            ))),
            _ => None,
        }
    }
}

