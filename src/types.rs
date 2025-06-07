use std::fmt::{Display, Formatter};

use rand::{seq::SliceRandom, thread_rng};

use crate::error::Error;

pub enum HandResult {
    Blackjack,
    Win,
    Lose,
    Push,
    NotFinished,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerChoice {
    Hit,
    Stand,
    Double,
    Split,
    Surrender,
}

impl PlayerChoice {
    pub fn parse_choice(input: &str, choices: PlayerChoices) -> Result<Self, Error> {
        let choice = match input.trim().to_uppercase().as_str() {
            "H" | "1" => PlayerChoice::Hit,
            "S" | "2" => PlayerChoice::Stand,
            "D" | "3" => PlayerChoice::Double,
            "P" | "4" => PlayerChoice::Split,
            "R" | "5" => PlayerChoice::Surrender,
            x => return Err(Error::InvalidInput(x.to_string())),
        };
        if choices.contains(choice.into()) {
            Ok(choice)
        } else {
            Err(Error::InvalidChoice(choice))
        }
    }
}

impl std::fmt::Display for PlayerChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerChoice::Hit => write!(f, "Hit"),
            PlayerChoice::Stand => write!(f, "Stand"),
            PlayerChoice::Double => write!(f, "Double Down"),
            PlayerChoice::Split => write!(f, "Split"),
            PlayerChoice::Surrender => write!(f, "Surrender"),
        }
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct PlayerChoices: u8 {
        const HIT = 0b0001;
        const STAND = 0b0010;
        const DOUBLE = 0b0100;
        const SPLIT = 0b1000;
        const SURRENDER = 0b0001_0000;
    }
}

impl From<PlayerChoice> for PlayerChoices {
    fn from(choice: PlayerChoice) -> Self {
        match choice {
            PlayerChoice::Hit => PlayerChoices::HIT,
            PlayerChoice::Stand => PlayerChoices::STAND,
            PlayerChoice::Double => PlayerChoices::DOUBLE,
            PlayerChoice::Split => PlayerChoices::SPLIT,
            PlayerChoice::Surrender => PlayerChoices::SURRENDER,
        }
    }
}

pub struct Deck {
    pub cards: Vec<Card>,
    pub count: isize
}

impl Deck {
    /// Creates a shoe of cards with the specified number of decks.
    pub fn create_shoe(decks: usize) -> Self {
        let mut cards = Vec::new();
        for _ in 0..decks {
            let deck = Deck::default();
            cards.extend(deck.cards);
        }
        cards.shuffle(&mut thread_rng());
        let mut this = Self { cards, count: 0 };
        this.place_cut_card();
        this
    }

    pub fn deal_hand(&mut self, hand_size: usize, dealer: bool) -> (Hand, bool) {
        let mut reshuffle_deck = false;
        let mut hand = Hand::new(dealer);
        for _ in 0..hand_size {
            let (card, cut_card) = self.draw();
            reshuffle_deck |= cut_card;
            hand.push(card);
        }
        hand.cards.reverse(); // Reverse to maintain the order of dealing

        (hand, reshuffle_deck)
    }
    /// Draw a card and bool is true if the cut card was drawn
    pub fn draw(&mut self) -> (Card, bool) {
        let card = self.cards.pop().unwrap();
        let card_count = card.count();
        self.count += card_count; // Update the count based on the card drawn

        if card.cut_card {
            // Take the next card after the cut card
            (self.cards.pop().unwrap(), true)
        } else {
            (card, false)
        }
    }
    /// Assumes new cards were just dealt
    fn place_cut_card(&mut self) {
        let total_cards = self.cards.len();
        let decks = total_cards / 52;
        let position = match decks {
            0 => panic!("Cannot place cut card in a deck with no cards"),
            // 3/4 of the deck
            1 => total_cards - 39,
            // Leave about half a deck
            2 => total_cards - 26,
            // 3-5 decks leave 1 deck
            3..=5 => total_cards - 52,
            // 6+ decks leave 1.5 decks
            _ => total_cards - 78,
        };
        let random_offset = (rand::random::<isize>() % 11) - 5; // Random offset between -5 and 5
        let cut_card_position = (position as isize + random_offset) as usize;
        let cut_card_position = total_cards - cut_card_position;

        self.cards.insert(cut_card_position, Card {
            suit: Suit::Spades, // Cut card doesn't have a suit
            face: CardFace::Face(Face::King), // Just a placeholder
            cut_card: true,
        });
    }
}

impl std::ops::Add for Deck {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut cards = self.cards;
        cards.extend(other.cards);

        Self { cards, count: self.count + other.count }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Hand {
    pub cards: Vec<Card>,
    pub hide_card: bool,
}

impl Hand {
    pub fn new(dealer_hand: bool) -> Self {
        Self {
            cards: Vec::new(),
            hide_card: dealer_hand,
        }
    }

    pub fn value(&self) -> u8 {
        let mut value = 0;
        let mut aces = 0;
        for card in self.cards.iter() {
            match card.face {
                CardFace::Ace => aces += 1,
                CardFace::Number(n) => value += n,
                CardFace::Face(_) => value += 10,
            }
        }
        for _ in 0..aces {
            if value + 11 <= 21 {
                value += 11;
            } else {
                value += 1;
            }
        }
        value
    }
    pub fn is_soft(&self) -> bool {
        let mut value = 0;
        let mut aces = 0;
        let mut aces_as_eleven = 0;
        
        // Calculate non-ace values first
        for card in self.cards.iter() {
            match card.face {
                CardFace::Ace => aces += 1,
                CardFace::Number(n) => value += n,
                CardFace::Face(_) => value += 10,
            }
        }
        
        // Add aces, tracking how many are counted as 11
        for _ in 0..aces {
            if value + 11 <= 21 {
                value += 11;
                aces_as_eleven += 1;
            } else {
                value += 1;
            }
        }
        
        // Hand is soft if at least one ace is counted as 11
        aces_as_eleven > 0
    }
    fn show_value(&self) -> u8 {
        if self.hide_card {
            return self.dealer_show_card().face_value();
        }
        self.value()
    }

    pub fn is_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.value() == 21 && self.cards.iter().any(|card| card.face == CardFace::Ace)
    }
    pub fn is_bust(&self) -> bool {
        self.value() > 21
    }

    pub fn can_split(&self) -> bool {
        if self.cards.len() != 2 {
            return false;
        }
        &self.cards[0] == &self.cards[1]
    }

    pub fn split(mut self) -> (Self, Self) {
        let card = self.cards.pop().unwrap();
        let hand1 = Hand { cards: vec![card.clone()], hide_card: self.hide_card };
        let hand2 = Hand { cards: vec![card], hide_card: self.hide_card };
        (hand1, hand2)
    }

    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }
    pub fn dealer_show_card(&self) -> &Card {
        &self.cards[1]
    }
}

impl Default for Deck {
    fn default() -> Self {
        let mut cards = Vec::new();
        for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades].iter() {
            for face in [
                CardFace::Ace,
                CardFace::Number(2),
                CardFace::Number(3),
                CardFace::Number(4),
                CardFace::Number(5),
                CardFace::Number(6),
                CardFace::Number(7),
                CardFace::Number(8),
                CardFace::Number(9),
                CardFace::Number(10),
                CardFace::Face(Face::Jack),
                CardFace::Face(Face::Queen),
                CardFace::Face(Face::King),
            ]
            .iter()
            {
                cards.push(Card {
                    suit: suit.clone(),
                    face: face.clone(),
                    cut_card: false,
                });
            }
        }
        Self { cards, count: 0 }
    }
}

#[derive(Clone, Debug)]
pub struct Card {
    pub suit: Suit,
    pub face: CardFace,
    pub cut_card: bool,
}

impl Card {
    pub fn face_value(&self) -> u8 {
        match self.face {
            CardFace::Ace => 11, // Ace is worth 11 by default
            CardFace::Number(n) => n,
            CardFace::Face(_) => 10, // Face cards are worth 10
        }
    }
    pub fn is_blackjack_card(&self) -> bool {
        self.face == CardFace::Ace || matches!(self.face, CardFace::Face(_))
    }
    pub fn count(&self) -> isize {
        if self.cut_card {
            0 // Cut card does not count towards the deck
        } else {
            match self.face {
                CardFace::Ace => -1,
                CardFace::Number(2..=6) => 1,
                CardFace::Number(7..=9) => 0, // Number cards 7-10 have 2 copies
                CardFace::Number(10) => -1, // 10 has 4 copies
                CardFace::Face(_) => -1, // Each face card has 4 copies
                CardFace::Number(_) => panic!("Unexpected card number"),
            }
        }
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.face == other.face
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Clone, PartialEq, Debug)]
pub enum CardFace {
    Ace,
    Number(u8),
    Face(Face),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Face {
    Jack,
    Queen,
    King,
}


impl Display for Hand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.cards.is_empty() {
            write!(f, "[Empty hand]")
        } else {
            let (cards_str, value) = if self.hide_card {
                // Dealer's hand shows only one card
                let dealer_card = self.dealer_show_card();
                let val = dealer_card.to_string();
                let empty_card = "??".to_string();
                let value = self.show_value();
                (vec![val, empty_card], value.to_string())
            } else {
                let cards_str = self.cards.iter()
                    .filter(|card| !card.cut_card)
                    .map(|card| card.to_string())
                    .collect();
                let value = self.value();
                let value = if self.is_blackjack() {
                    "Blackjack".to_string()
                } else if self.is_soft() {
                    format!("Soft {}", value)
                } else {
                    value.to_string()
                };
                (cards_str, value)
            };

            write!(f, "[{}] (Value: {})", cards_str.join(", "), value)
        }
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.cut_card {
            write!(f, "CUT")
        } else {
            write!(f, "{}{}", self.face, self.suit)
        }
    }
}

impl Display for CardFace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CardFace::Ace => write!(f, "A"),
            CardFace::Number(n) => write!(f, "{}", n),
            CardFace::Face(face) => write!(f, "{}", face),
        }
    }
}

impl Display for Face {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Face::Jack => write!(f, "J"),
            Face::Queen => write!(f, "Q"),
            Face::King => write!(f, "K"),
        }
    }
}

impl Display for Suit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Suit::Hearts => write!(f, "♥"),
            Suit::Diamonds => write!(f, "♦"),
            Suit::Clubs => write!(f, "♣"),
            Suit::Spades => write!(f, "♠"),
        }
    }
}