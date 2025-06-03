use std::fmt::Write;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use blackjack_core::card::hand::Status;
use blackjack_core::state::GameState;

use crate::app::App;
use crate::input::InputField;

pub fn display(frame: &mut Frame, app: &App) {
    let columns =
        Layout::horizontal(Constraint::from_percentages([25, 50, 25])).split(frame.area());
    draw_games_list(frame, app, columns[0]);
    draw_middle_zone(frame, app, columns[1]);
    draw_statistics_section(frame, app, columns[2]);
}

fn draw_games_list(frame: &mut Frame, app: &App, area: Rect) {
    let list = app.games.iter().enumerate().fold(
        String::with_capacity(5 * app.games.len()),
        |mut output, (i, _)| {
            let prefix = if i == app.selected_game { " > " } else { "   " };
            writeln!(output, "{prefix}{i}").unwrap();
            output
        },
    );
    let content = Paragraph::new(list).block(Block::default().title("Games").borders(Borders::ALL));
    frame.render_widget(content, area);
}

fn draw_statistics_section(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title("Statistics").borders(Borders::ALL);
    if let Some(current_game) = app.current_game() {
        let content = Paragraph::new(format!("{}", current_game.table.statistics)).block(block);
        frame.render_widget(content, area);
    } else {
        frame.render_widget(block, area);
    }
}

fn draw_middle_zone(frame: &mut Frame, app: &App, area: Rect) {
    let middle_rows = Layout::vertical(Constraint::from_percentages([75, 25])).split(area);
    draw_game(frame, app, middle_rows[0]);
    draw_input_area(frame, app, middle_rows[1]);
}

fn draw_input_area(frame: &mut Frame, app: &App, area: Rect) {
    let content = app.current_game().map_or_else(
        || "No game selected".to_string(),
        |current_game| {
            let text = current_game
                .input_field
                .as_ref()
                .map_or_else(String::new, |input_field| match input_field {
                    InputField::PlaceBet(s) => format!("Enter your bet: {s}"),
                    InputField::PlaceInsuranceBet(s) => {
                        format!("Place an insurance bet? Enter bet or 0: {s}")
                    }
                    InputField::ChooseSurrender => "Surrender? (y) or (n)".to_string(),
                    InputField::PlayHand(actions) => {
                        let mut output = String::with_capacity(actions.len() * 7);
                        for action in actions {
                            write!(output, "{action:?}, ").unwrap();
                        }
                        output
                    }
                });
            let last_error = current_game
                .last_error
                .as_ref()
                .map_or_else(String::new, |e| format!("{e}!"));
            format!(
                "{text}\nChips: {chips}\n{last_error}",
                chips = current_game.table.chips
            )
        },
    );
    let content =
        Paragraph::new(content).block(Block::default().title("Input").borders(Borders::ALL));
    frame.render_widget(content, area);
}

fn draw_game(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title_top(
            (Line::styled("BLACKJACK", Style::default().bold().red())).alignment(Alignment::Center),
        )
        .borders(Borders::ALL);
    if let Some(current_game) = app.current_game() {
        let content = Paragraph::new(game_text(&current_game.game_state)).block(block);
        frame.render_widget(content, area);
    } else {
        frame.render_widget(block, area);
    }
}

#[allow(clippy::too_many_lines)]
fn game_text(game_state: &GameState) -> String {
    match game_state {
        GameState::Betting => "Place your bet!".to_string(),
        GameState::DealFirstPlayerCard { bet } => {
            format!("DealFirstPlayerCard\nBet: {bet}\n")
        }
        GameState::DealFirstDealerCard { player_hand } => {
            format!("DealFirstDealerCard\nPlayer: {}\n", player_hand.value)
        }
        GameState::DealSecondPlayerCard {
            player_hand,
            dealer_hand,
        } => {
            format!(
                "DealSecondPlayerCard\nPlayer: {}\nDealer showing: {}\n",
                player_hand.value,
                dealer_hand.showing()
            )
        }
        GameState::DealHoleCard {
            player_hand,
            dealer_hand,
        } => {
            format!(
                "DealHoleCard\nPlayer: {}\nDealer showing: {}\n",
                player_hand.value,
                dealer_hand.showing()
            )
        }
        GameState::OfferEarlySurrender {
            player_hand,
            dealer_hand,
        } => {
            format!(
                "OfferEarlySurrender\nPlayer: {}\nDealer showing: {}\n",
                player_hand.value,
                dealer_hand.showing()
            )
        }
        GameState::OfferInsurance {
            player_hand,
            dealer_hand,
        } => {
            format!(
                "OfferInsurance\nPlayer: {}\nDealer showing: {}\n",
                player_hand.value,
                dealer_hand.showing(),
            )
        }
        GameState::CheckDealerHoleCard {
            player_hand,
            dealer_hand,
            insurance_bet: insurance,
        } => {
            format!(
                "CheckDealerHoleCard\nPlayer: {}\nDealer showing: {}\n{}\n",
                player_hand.value,
                dealer_hand.showing(),
                if *insurance > 0 {
                    format!("Insurance: {insurance}")
                } else {
                    String::new()
                },
            )
        }
        GameState::PlayPlayerTurn {
            player_turn,
            dealer_hand,
            ..
        } => {
            format!(
                "PlayPlayerTurn\nPlayer: {}\nDealer showing: {}",
                player_turn.current_hand().value,
                dealer_hand.showing(),
            )
        }
        GameState::PlayerStand {
            player_turn,
            dealer_hand,
            ..
        } => {
            format!(
                "Stand\nPlayer: {}\nDealer showing: {}",
                player_turn.current_hand().value,
                dealer_hand.showing(),
            )
        }
        GameState::PlayerHit {
            player_turn,
            dealer_hand,
            ..
        } => {
            format!(
                "Hit\nPlayer: {}\nDealer showing: {}",
                player_turn.current_hand().value,
                dealer_hand.showing(),
            )
        }
        GameState::PlayerDouble {
            player_turn,
            dealer_hand,
            ..
        } => {
            format!(
                "Double\nPlayer: {}\nDealer showing: {}",
                player_turn.current_hand().value,
                dealer_hand.showing(),
            )
        }
        GameState::PlayerSplit {
            player_turn,
            dealer_hand,
            ..
        } => {
            format!(
                "Split\nPlayer: {}\nDealer showing: {}",
                player_turn.current_hand().value,
                dealer_hand.showing(),
            )
        }
        GameState::DealFirstSplitCard {
            player_turn,
            new_hand,
            dealer_hand,
            ..
        } => {
            format!(
                "DealFirstSplitCard\nPlayer: {}\nNew Hand: {}\nDealer showing: {}",
                player_turn.current_hand().value,
                new_hand.value,
                dealer_hand.showing(),
            )
        }
        GameState::DealSecondSplitCard {
            player_turn,
            new_hand,
            dealer_hand,
            ..
        } => {
            format!(
                "DealSecondSplitCard\nPlayer: {}\nNew Hand: {}\nDealer showing: {}",
                player_turn.current_hand().value,
                new_hand.value,
                dealer_hand.showing(),
            )
        }
        GameState::PlayerSurrender {
            player_turn,
            dealer_hand,
            ..
        } => {
            format!(
                "Surrender\nPlayer: {}\nDealer showing: {}",
                player_turn.current_hand().value,
                dealer_hand.showing(),
            )
        }
        GameState::RevealHoleCard {
            finished_hands,
            dealer_hand,
            ..
        } => {
            format!(
                "The dealer reveals his hole card...\nPlayer: {}\nDealer showing: {}",
                finished_hands.iter().fold(
                    String::with_capacity(finished_hands.len() * 7),
                    |mut output, h| {
                        let value = &h.value;
                        write!(output, "{value}, ").unwrap();
                        output
                    }
                ),
                dealer_hand.showing(),
            )
        }
        GameState::PlayDealerTurn {
            finished_hands,
            dealer_hand,
            ..
        } => {
            format!(
                "PlayDealerTurn\nPlayer: {}\nDealer: {}",
                finished_hands.iter().fold(
                    String::with_capacity(finished_hands.len() * 7),
                    |mut output, h| {
                        let value = &h.value;
                        write!(output, "{value}, ").unwrap();
                        output
                    }
                ),
                dealer_hand.value,
            )
        }
        GameState::RoundOver {
            finished_hands,
            dealer_hand,
            ..
        } => {
            let announcement = match &dealer_hand.status {
                Status::Blackjack => "Dealer has blackjack!".to_string(),
                Status::Bust => "Dealer busts!".to_string(),
                Status::Stood => format!("Dealer has {}.", dealer_hand.value.total),
                _ => unreachable!("dealer hand should not be in play or surrendered"),
            };
            format!(
                "{}\nPlayer: {}\nDealer: {}",
                announcement,
                finished_hands.iter().fold(
                    String::with_capacity(finished_hands.len() * 7),
                    |mut output, h| {
                        let value = &h.value;
                        write!(output, "{value}, ").unwrap();
                        output
                    }
                ),
                dealer_hand.value,
            )
        }
        GameState::Payout {
            total_bet,
            total_winnings,
        } => {
            let difference = i64::from(*total_winnings) - i64::from(*total_bet);
            match difference {
                1.. => format!("You win {total_winnings} chips (+{difference})!"),
                0 => format!("You make back {total_winnings} chips. You push!"),
                _ if *total_winnings > 0 => {
                    format!("You make back {total_winnings} out of {total_bet} chips!")
                }
                _ => format!("You lose {} chips!", difference.abs()),
            }
        }
        GameState::Shuffle => "Shuffling the shoe...".to_string(),
        GameState::GameOver => "Game Over!".to_string(),
    }
}
