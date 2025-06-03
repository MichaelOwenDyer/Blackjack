//! This module contains the types and functions for working with cards in a game of blackjack.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl fmt::Display for Suit {
    /// Suits are displayed as their name, e.g. "Clubs", "Diamonds", "Hearts", "Spades"
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clubs => write!(f, "Clubs"),
            Self::Diamonds => write!(f, "Diamonds"),
            Self::Hearts => write!(f, "Hearts"),
            Self::Spades => write!(f, "Spades"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl fmt::Display for Rank {
    /// Ranks are displayed as "a Rank", e.g. "a Two", "a Seven", "an Eight", "an Ace"
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Two => write!(f, "a Two"),
            Self::Three => write!(f, "a Three"),
            Self::Four => write!(f, "a Four"),
            Self::Five => write!(f, "a Five"),
            Self::Six => write!(f, "a Six"),
            Self::Seven => write!(f, "a Seven"),
            Self::Eight => write!(f, "an Eight"),
            Self::Nine => write!(f, "a Nine"),
            Self::Ten => write!(f, "a Ten"),
            Self::Jack => write!(f, "a Jack"),
            Self::Queen => write!(f, "a Queen"),
            Self::King => write!(f, "a King"),
            Self::Ace => write!(f, "an Ace"),
        }
    }
}

impl Rank {
    /// Returns how much a card with this rank is worth in the game.
    /// All face cards are worth 10, and aces are worth 11.
    #[must_use]
    pub const fn worth(&self) -> u8 {
        match self {
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
            Self::Ten | Self::Jack | Self::Queen | Self::King => 10,
            Self::Ace => 11,
        }
    }
}

/// A card is a combination of a rank and a suit.
/// Copy is intentionally not derived to reflect the nature of physical cards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl fmt::Display for Card {
    /// Cards are displayed as "a Rank of Suit", e.g. "a Two of Clubs"
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} of {}", self.rank, self.suit)
    }
}

impl Card {
    /// Returns the card corresponding to the given ordinal value (0-51).
    /// The ordinal value is the index of the card in a deck sorted by rank and then suit,
    /// e.g. twos first, then threes, fours, etc.
    ///
    /// # Panics
    ///
    /// Panics if `ordinal` is >= 52
    fn from_ordinal(ordinal: usize) -> Self {
        let rank = match ordinal / 4 {
            0 => Rank::Two,
            1 => Rank::Three,
            2 => Rank::Four,
            3 => Rank::Five,
            4 => Rank::Six,
            5 => Rank::Seven,
            6 => Rank::Eight,
            7 => Rank::Nine,
            8 => Rank::Ten,
            9 => Rank::Jack,
            10 => Rank::Queen,
            11 => Rank::King,
            12 => Rank::Ace,
            _ => panic!("Invalid ordinal {}", ordinal),
        };
        let suit = match ordinal % 4 {
            0 => Suit::Clubs,
            1 => Suit::Diamonds,
            2 => Suit::Hearts,
            3 => Suit::Spades,
            _ => unreachable!(),
        };
        Self { rank, suit }
    }
}

pub mod hand {
    use std::cmp::Ordering;
    use std::fmt;
    use std::ops::AddAssign;

    use crate::card::{Card, Rank};
    use crate::rules::{BlackjackPayout, DealerSoft17Action};

    /// Represents the game value of a hand, e.g. "Soft 20"
    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct Value {
        /// Whether the hand has an ace that is currently worth 11
        pub soft: bool,
        /// The total value of the hand
        pub total: u8,
    }

    impl From<&Card> for Value {
        /// Converts a card into a hand value.
        fn from(card: &Card) -> Self {
            Self {
                soft: card.rank == Rank::Ace,
                total: card.rank.worth(),
            }
        }
    }

    impl<T: Into<Value>> AddAssign<T> for Value {
        /// Adds two hand values together, taking care to handle soft values and avoid busting if possible
        fn add_assign(&mut self, rhs: T) {
            let Self {
                mut soft,
                total: mut worth,
            } = rhs.into();
            // Prevent busting by converting the soft ace to a hard ace
            if soft && self.total + worth > 21 {
                worth -= 10; // Convert the ace from 11 to 1
                soft = false;
            }
            // Prevent busting by converting the current hand's soft ace to a hard ace
            if self.soft && self.total + worth > 21 {
                self.total -= 10; // Convert the ace from 11 to 1
                self.soft = false;
            }
            self.total += worth; // Add the card's worth to the total
            self.soft |= soft; // If either hand has a soft ace, the result is a soft hand
        }
    }

    impl fmt::Display for Value {
        /// A hand is displayed as "Soft/Hard total", e.g. "Soft 20"
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{} {}",
                if self.soft { "Soft" } else { "Hard" },
                self.total
            )
        }
    }

    /// Represents the status of a hand.
    /// A hand may still be in play, or it may be in any of the four terminal states.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub enum Status {
        #[default]
        InPlay,
        Stood,
        Bust,
        Blackjack,
        Surrendered,
    }

    /// Represents the dealer's hand.
    #[derive(Debug, PartialEq, Eq)]
    pub struct DealerHand {
        /// The value of this hand
        pub value: Value,
        /// The status of this hand
        pub status: Status,
        /// The cards in this hand (non-empty at all times)
        cards: Vec<Card>,
        /// Whether the dealer stands or hits on soft 17
        soft_17_action: DealerSoft17Action,
    }

    impl AddAssign<Card> for DealerHand {
        /// Adds a card to the dealer's hand.
        fn add_assign(&mut self, rhs: Card) {
            debug_assert_eq!(self.status, Status::InPlay, "cannot add to finished hand");
            self.value += &rhs;
            self.cards.push(rhs);
            self.status = match (self.value.soft, self.value.total) {
                (true, 17) if self.hits_on_soft_17() => Status::InPlay,
                (true, 21) if self.cards.len() == 2 => Status::Blackjack,
                (_, 17..=21) => Status::Stood,
                (_, 22..) => Status::Bust,
                _ => Status::InPlay,
            };
        }
    }

    impl DealerHand {
        /// Creates a new dealer hand with the given card and soft 17 behavior.
        #[must_use]
        pub fn new(card: Card, soft_17_action: DealerSoft17Action) -> Self {
            Self {
                value: Value::from(&card),
                status: Status::InPlay,
                cards: vec![card],
                soft_17_action,
            }
        }

        /// Returns the worth of the dealer's up card, which is what the player must base their decisions on.
        #[must_use]
        pub fn showing(&self) -> u8 {
            self.cards[0].rank.worth()
        }

        /// Returns whether the dealer hits on soft 17.
        #[must_use]
        pub fn hits_on_soft_17(&self) -> bool {
            self.soft_17_action == DealerSoft17Action::Hit
        }
    }

    /// Represents a hand of cards held by the player.
    #[derive(Debug, PartialEq, Eq)]
    pub struct PlayerHand {
        /// The player's bet on this hand
        pub bet: u32,
        /// The value of this hand
        pub value: Value,
        /// The status of this hand
        pub status: Status,
        /// The cards in this hand (non-empty at all times)
        pub cards: Vec<Card>,
        /// The player's winnings on this hand
        pub winnings: u32,
    }

    impl AddAssign<Card> for PlayerHand {
        /// Adds a card to the player's hand.
        fn add_assign(&mut self, rhs: Card) {
            debug_assert_eq!(self.status, Status::InPlay, "cannot add to finished hand");
            self.value += &rhs;
            self.cards.push(rhs);
            self.status = match self.value.total {
                22.. => Status::Bust,
                21 if self.size() == 2 => Status::Blackjack,
                21 => Status::Stood,
                _ => Status::InPlay,
            }
        }
    }

    impl PlayerHand {
        /// Creates a new player hand with the given card and bet.
        #[must_use]
        pub fn new(card: Card, bet: u32) -> Self {
            Self {
                bet,
                value: Value::from(&card),
                status: Status::InPlay,
                cards: vec![card],
                winnings: 0,
            }
        }

        /// The player stands on this hand.
        pub fn stand(&mut self) {
            debug_assert_eq!(self.status, Status::InPlay, "cannot stand on finished hand");
            self.status = Status::Stood;
        }

        /// The player doubles down on this hand.
        /// The bet is doubled, and the provided card is added to the hand.
        /// If the hand is not bust, the player stands.
        pub fn double(&mut self, card: Card) {
            debug_assert_eq!(
                self.size(),
                2,
                "cannot double down on hand with more than two cards"
            );
            debug_assert_eq!(
                self.status,
                Status::InPlay,
                "cannot double down on finished hand"
            );
            self.bet *= 2;
            *self += card;
            // If the hand is not finished otherwise, the player stands
            if self.status == Status::InPlay {
                self.status = Status::Stood;
            }
        }

        /// The player splits the hand into two hands. This hand must be a pair!
        /// The new hand has the same bet as the original hand.
        /// # Panics
        /// Will panic if the hand is not a pair.
        #[must_use]
        pub fn split(&mut self) -> Self {
            debug_assert!(self.is_pair(), "cannot split hand that is not a pair");
            let split_card = self.cards.pop().expect("Hand must be a pair"); // Remove the second card
            self.value = Value::from(&self.cards[0]); // The value of this hand is now the first card
            Self::new(split_card, self.bet) // Create a new hand with the second card
        }

        /// The player surrenders this hand.
        pub fn surrender(&mut self) {
            debug_assert_eq!(
                self.size(),
                2,
                "cannot surrender on hand with more than two cards"
            );
            self.status = Status::Surrendered;
        }

        /// Returns the number of cards in this hand.
        #[must_use]
        pub fn size(&self) -> usize {
            self.cards.len()
        }

        /// Returns whether this hand is a pair.
        /// A pair is a hand consisting of only two cards with equal rank.
        #[must_use]
        pub fn is_pair(&self) -> bool {
            self.size() == 2 && self.cards[0].rank == self.cards[1].rank
        }

        /// Calculates the winnings for this hand based on the dealer's hand.
        /// This method should only be called once the dealer's hand is in a terminal state.
        #[must_use]
        pub fn calculate_winnings(
            &self,
            dealer_hand: &DealerHand,
            blackjack_payout: BlackjackPayout,
        ) -> u32 {
            match (&self.status, &dealer_hand.status) {
                (Status::Surrendered, _) => self.payout_surrender(), // Player surrender
                (Status::Blackjack, Status::Blackjack) => self.payout_push(), // Blackjack push
                (Status::Blackjack, _) => self.payout_blackjack(blackjack_payout), // Blackjack win
                (_, Status::Blackjack) | (Status::Bust, _) => self.payout_loss(), // Dealer blackjack or player bust
                (_, Status::Bust) => self.payout_win(),                           // Dealer bust
                _ => match self.value.total.cmp(&dealer_hand.value.total) {
                    Ordering::Greater => self.payout_win(), // Player win
                    Ordering::Equal => self.payout_push(),  // Push
                    Ordering::Less => self.payout_loss(),   // Dealer win
                },
            }
        }

        /// Calculates the winnings for a blackjack win based on whether the game pays 3:2 or 6:5.
        const fn payout_blackjack(&self, payout: BlackjackPayout) -> u32 {
            match payout {
                BlackjackPayout::ThreeToTwo => self.bet + self.bet * 3 / 2,
                BlackjackPayout::SixToFive => self.bet + self.bet * 6 / 5,
            }
        }

        /// Calculates the winnings for a normal win, which is double the bet.
        const fn payout_win(&self) -> u32 {
            self.bet * 2
        }

        /// Calculates the winnings for a push, which is the same as the bet.
        const fn payout_push(&self) -> u32 {
            self.bet
        }

        /// Calculates the winnings for a surrender, which is half the bet.
        const fn payout_surrender(&self) -> u32 {
            self.bet / 2
        }

        /// Calculates the winnings for a loss, which is 0.
        const fn payout_loss(&self) -> u32 {
            0
        }
    }

    /// All the player's hands in a round of blackjack.
    /// This always starts with just one hand, but the player might split it into arbitrarily many.
    /// Split hands are pushed onto the vec.
    /// The player plays each hand in turn, and the hands are resolved in the order they were split.
    #[derive(Debug, PartialEq, Eq)]
    pub struct PlayerTurn {
        /// The hands in the player's turn, initially just their starting hand.
        /// This will only grow in size if the player splits.
        hands: Vec<PlayerHand>,
        /// The index of the hand the player is currently playing.
        /// u8 is more than sufficient for the number of hands the player could realistically split
        current_hand_index: usize,
    }

    /// Convenience implementation to convert a player hand into a player turn.
    impl From<PlayerHand> for PlayerTurn {
        fn from(hand: PlayerHand) -> Self {
            Self {
                hands: vec![hand],
                current_hand_index: 0,
            }
        }
    }

    impl PlayerTurn {
        /// Returns a mutable reference to the current hand.
        pub fn current_hand_mut(&mut self) -> &mut PlayerHand {
            &mut self.hands[self.current_hand_index]
        }
        /// Returns a reference to the current hand.
        pub fn current_hand(&self) -> &PlayerHand {
            &self.hands[self.current_hand_index]
        }
        /// Returns the total number of hands belonging to the player.
        pub fn hands(&self) -> u8 {
            self.hands.len() as u8
        }

        /// Adds a new (split) hand to the player's turn.
        /// The player may not play this hand immediately, so it is deferred until later.
        pub fn defer(&mut self, hand: PlayerHand) {
            self.hands.push(hand);
        }

        /// Continue playing on the next hand which is still in-play.
        /// If the current hand is still in-play, then nothing happens,
        /// otherwise the index is incremented until a hand is found that is still in-play.
        /// This ensures that finished hands are not played again, and we eventually
        /// play all hands to completion in the order they were split.
        /// If there are no more hands to play, Self is deconstructed and Err(hands) is returned.
        pub fn continue_playing(mut self) -> Result<Self, Vec<PlayerHand>> {
            if let Some(position) = self
                .hands
                .iter()
                .skip(self.current_hand_index)
                .position(|hand| hand.status == Status::InPlay)
            {
                self.current_hand_index += position;
                Ok(self)
            } else {
                Err(self.hands)
            }
        }
    }

    /// Tests whether a hand is composed of cards with the given values.
    /// The multiset of card values in the hand must be equal to the multiset of values provided.
    ///
    /// # Example
    /// ```
    /// use blackjack_core::card::{Card, Rank, Suit};
    /// use blackjack_core::card::hand::PlayerHand;
    /// use blackjack_core::composed;
    ///
    /// let mut hand = PlayerHand::new(Card { rank: Rank::Ten, suit: Suit::Clubs }, 100);
    /// hand += Card { rank: Rank::Five, suit: Suit::Diamonds };
    ///
    /// assert!(composed!(hand => 10, 5));
    /// assert!(composed!(hand => 5, 10));
    /// assert!(!composed!(hand => 5));
    /// assert!(!composed!(hand => 10));
    /// assert!(!composed!(hand => 10, 5, 5));
    /// assert!(composed!(hand => 9, 5; 10, 5));
    /// assert!(composed!(hand => 10, 5; 9, 5));
    /// ```
    #[macro_export]
    macro_rules! composed {
        ($hand:ident => $($x:expr),+) => ({
            let mut values: Vec<u8> = $hand.cards.iter().map(|card| card.rank.worth()).collect();
            true $(&& match values.iter().position(|&val| val == $x) {
                Some(pos) => {
                    values.swap_remove(pos);
                    true
                },
                None => false,
            })* && values.is_empty()
        });
        ($hand:ident => $($($x:expr),+);+) => ({
            false $(|| composed!($hand => $($x),*))*
        });
    }
}

pub mod shoe {
    use rand::rng;
    use rand_distr::{weighted::WeightedTreeIndex, Distribution};

    use crate::card::Card;

    /// A shoe is a container that contains multiple decks of cards.
    #[derive(Debug, Clone)]
    pub struct Shoe {
        /// The number of decks in the shoe
        pub decks: u8,
        /// The number of cards that have been drawn from the shoe
        pub cards_drawn: u16,
        /// The proportion of cards to play before shuffling
        pub max_penetration: f32,
        /// Weighted distribution to draw random cards from the shoe without replacement.
        dist: WeightedTreeIndex<u8>,
    }

    impl Shoe {
        /// Create a new shoe with the given number of decks and shuffle threshold.
        /// The shoe is initialized with all cards present.
        /// # Panics
        ///
        /// Panics if the number of decks is 0
        #[must_use]
        pub fn new(decks: u8, shuffle_threshold: f32) -> Self {
            Self {
                decks,
                cards_drawn: 0,
                max_penetration: shuffle_threshold,
                dist: WeightedTreeIndex::new([decks; 52]).unwrap(),
            }
        }

        /// Draws a random card from the shoe.
        /// The card is removed from the shoe, and the distribution is updated to reflect the new weight.
        /// If the last card is drawn, the shoe is shuffled.
        pub fn draw_card(&mut self) -> Card {
            let ordinal = self.dist.sample(&mut rng());
            self.cards_drawn += 1;
            let new_weight = self.dist.get(ordinal) - 1;
            // Update the distribution to reflect the new weight of the removed card
            if self.dist.update(ordinal, new_weight).is_err() {
                // The update failed, so we must have drawn the last card
                debug_assert_eq!(self.cards_drawn, self.decks as u16 * 52, "last card drawn");
                self.shuffle();
            }
            Card::from_ordinal(ordinal)
        }

        /// Checks if the shoe needs to be shuffled.
        #[must_use]
        pub fn needs_shuffle(&self) -> bool {
            let penetration = f32::from(self.cards_drawn) / f32::from(self.decks as u16 * 52);
            penetration >= self.max_penetration
        }

        /// Shuffles the shoe.
        /// All cards are returned to the shoe, and the distribution is updated to reflect the new weights.
        ///
        /// # Panics
        ///
        /// Panics if the number of decks is 0
        pub fn shuffle(&mut self) {
            self.cards_drawn = 0;
            self.dist = WeightedTreeIndex::new([self.decks; 52]).unwrap();
        }
    }
}
