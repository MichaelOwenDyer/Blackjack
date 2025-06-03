use crossterm::event::KeyCode;

use blackjack_core::game::{HandAction, Input, Table};
use blackjack_core::state::GameState;

#[derive(Debug)]
pub enum InputField {
    PlaceBet(String),
    PlaceInsuranceBet(String),
    ChooseSurrender,
    PlayHand(Vec<HandAction>),
}

impl InputField {
    pub fn from_game(state: &GameState, table: &Table) -> Option<Self> {
        match state {
            GameState::Betting => Some(Self::PlaceBet(String::new())),
            GameState::OfferInsurance { .. } => Some(Self::PlaceInsuranceBet(String::new())),
            GameState::OfferEarlySurrender { .. } => Some(Self::ChooseSurrender),
            GameState::PlayPlayerTurn { player_turn, .. } => {
                let mut allowed_actions = Vec::with_capacity(5);
                allowed_actions.push(HandAction::Hit);
                allowed_actions.push(HandAction::Stand);
                if table.check_double_allowed(player_turn).is_ok() {
                    allowed_actions.push(HandAction::Double);
                }
                if table.check_split_allowed(player_turn).is_ok() {
                    allowed_actions.push(HandAction::Split);
                }
                if table
                    .check_surrender_allowed(player_turn.current_hand())
                    .is_ok()
                {
                    allowed_actions.push(HandAction::Surrender);
                }
                Some(Self::PlayHand(allowed_actions))
            }
            _ => None,
        }
    }

    pub fn consider(&mut self, key_code: KeyCode) -> Option<Input> {
        match self {
            Self::PlaceBet(s) => parse_bet_from_string(key_code, s),
            Self::PlaceInsuranceBet(s) => parse_bet_from_string(key_code, s),
            Self::ChooseSurrender => select_choice(key_code),
            Self::PlayHand(_) => select_action(key_code),
        }
    }
}

fn parse_bet_from_string(key: KeyCode, field: &mut String) -> Option<Input> {
    if key == KeyCode::Enter {
        if let Ok(bet) = field.parse() {
            return Some(Input::Bet(bet));
        }
    }
    match key {
        KeyCode::Char(c) => field.push(c),
        KeyCode::Backspace => {
            field.pop();
        }
        _ => {}
    }
    None
}

const fn select_choice(key: KeyCode) -> Option<Input> {
    match key {
        KeyCode::Char('y' | 'Y') => Some(Input::Choice(true)),
        KeyCode::Char('n' | 'N') => Some(Input::Choice(false)),
        _ => None,
    }
}

fn select_action(key: KeyCode) -> Option<Input> {
    match key {
        KeyCode::Char('h' | 'H') => Some(Input::Action(HandAction::Hit)),
        KeyCode::Char('s' | 'S') => Some(Input::Action(HandAction::Stand)),
        KeyCode::Char('d' | 'D') => Some(Input::Action(HandAction::Double)),
        KeyCode::Char('p' | 'P') => Some(Input::Action(HandAction::Split)),
        KeyCode::Char('r' | 'R') => Some(Input::Action(HandAction::Surrender)),
        _ => None,
    }
}
