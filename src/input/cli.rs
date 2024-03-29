use std::fmt::{Display, Formatter};
use std::{fmt, io};
use std::str::FromStr;
use crate::card::hand::{DealerHand, PlayerHand};
use crate::game::Game;
use crate::input::{HandAction, GameAction};

pub fn place_bet_or_quit(game: &Game, chips: u32) -> GameAction {
    println!("You have {chips} chips. How many chips would you like to bet? Type \"stop\" to quit.");
    let mut input = String::new();
    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let trimmed = input.trim();
        if trimmed == "stop" {
            return GameAction::Quit;
        }
        match trimmed.parse() {
            Ok(0) => println!("You must bet at least 1 chip!"),
            Ok(bet) if bet > chips => println!("You don't have enough chips!"),
            Ok(bet) => match (game.max_bet, game.min_bet) {
                (Some(max), _) if bet > max => println!("You cannot bet more than {max} chips!"),
                (_, Some(min)) if bet < min => println!("You cannot bet fewer than {min} chips!"),
                _ => return GameAction::Bet(bet),
            },
            Err(_) => println!("Please enter a number!"),
        }
        input.clear();
    }
}

pub fn surrender_early(_: &Game, _: &PlayerHand, _: &DealerHand) -> bool {
    println!("Would you like to surrender before the dealer checks for blackjack? (y/n)");
    let mut input = String::new();
    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match input.trim() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please enter y or n!"),
        }
        input.clear();
    }
}

pub fn offer_insurance(max_bet: u32) -> u32 {
    println!("Would you like to place an insurance bet? Enter your bet or 0 to decline.");
    let mut input = String::new();
    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match input.trim().parse() {
            Ok(0) => return 0,
            Ok(bet) if bet > max_bet => println!("You cannot bet more than half your original bet!"),
            Ok(bet) => {
                println!("You place an insurance bet of {bet} chips.");
                return bet;
            },
            Err(_) => println!("Please enter a number!"),
        }
        input.clear();
    }
}

/// Prompts the player to make a move
/// Which actions are available depends on the number of cards in the hand,
/// whether the hand is a pair, and whether the player has enough chips to double their bet
pub fn get_hand_action(game: &Game, player_hand: &PlayerHand, _: &DealerHand, chips: u32) -> HandAction {
    let is_pair = player_hand.is_pair();
    let two_cards = is_pair || player_hand.cards.len() == 2;
    let can_double_bet = chips >= player_hand.bet;
    let can_double_after_split = player_hand.splits == 0 || game.double_after_split;
    let can_split_again = game.max_splits.map_or(true, |max| player_hand.splits < max);
    let can_split_aces = game.split_aces || !is_pair || !player_hand.value.soft;
    let can_surrender = game.late_surrender;
    let mut allowed_moves = Vec::with_capacity(5);
    allowed_moves.push(HandAction::Hit);
    allowed_moves.push(HandAction::Stand);
    if two_cards && can_double_bet && can_double_after_split {
        allowed_moves.push(HandAction::Double);
    }
    if is_pair && can_double_bet && can_split_again && can_split_aces {
        allowed_moves.push(HandAction::Split);
    }
    if can_surrender {
        allowed_moves.push(HandAction::Surrender);
    }
    let allowed_moves = allowed_moves
        .into_iter()
        .fold(String::with_capacity(75), |mut acc, action| {
            let formatted = format!("{:15}", format!("{}", action));
            acc.push_str(&formatted);
            acc
        });
    if allowed_moves.capacity() != 75 {
        panic!("Capacity of allowed_moves is {}!", allowed_moves.capacity());
    }
    println!("What would you like to do?\n{}", allowed_moves);

    let mut input = String::new();
    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match input.trim().parse() {
            Ok(action) => match action {
                HandAction::Double if !two_cards => println!("You can only double down on your first two cards!"),
                HandAction::Double if !can_double_bet => println!("You don't have enough chips to double down!"),
                HandAction::Double if !can_double_after_split => println!("You can't double down after splitting!"),
                HandAction::Split if !is_pair => println!("You can only split a pair!"),
                HandAction::Split if !can_double_bet => println!("You don't have enough chips to split!"),
                HandAction::Split if !can_split_again => println!("You can't split again!"),
                HandAction::Split if !can_split_aces => println!("You can't split aces!"),
                HandAction::Surrender if can_surrender => println!("You can't surrender!"),
                action => return action,
            },
            Err(_) => println!("Please enter a valid action!"),
        };
        input.clear();
    }
}

impl Display for HandAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HandAction::Stand => write!(f, "Stand (s)"),
            HandAction::Hit => write!(f, "Hit (h)"),
            HandAction::Double => write!(f, "Double (d)"),
            HandAction::Split => write!(f, "Split (p)"),
            HandAction::Surrender => write!(f, "Surrender (u)"),
        }
    }
}

impl FromStr for HandAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "s" | "stand" => Ok(HandAction::Stand),
            "h" | "hit" => Ok(HandAction::Hit),
            "d" | "double" => Ok(HandAction::Double),
            "p" | "split" => Ok(HandAction::Split),
            "u" | "surrender" => Ok(HandAction::Surrender),
            _ => Err(()),
        }
    }
}