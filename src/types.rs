use rand::{seq::SliceRandom, thread_rng};


pub enum PlayerChoice {
    Hit,
    Stand,
    Double,
    Split,
}

pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn multiple(decks: usize) -> Self {
        let mut cards = Vec::new();
        for _ in 0..decks {
            let deck = Deck::default();
            cards.extend(deck.cards);
        }
        cards.shuffle(&mut thread_rng());
        Self { cards }
    }
    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        self.cards.shuffle(&mut thread_rng());
    }

    pub fn deal_hands(&mut self, n: usize) -> Vec<Hand>{
        let mut hands: Vec<Hand> = Vec::new();
        for _ in 0..n {
            for hand in hands.iter_mut() {
                hand.push(self.cards.pop().unwrap());
            }
        }
        hands
    }
}

impl std::ops::Add for Deck {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut cards = self.cards;
        cards.extend(other.cards);
        Self { cards }
    }
}

#[derive(Clone)]
pub struct Hand {
    pub cards: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        Self { cards: Vec::new() }
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

    pub fn split(mut self) -> (Self, Self) {
        let card = self.cards.pop().unwrap();
        let hand1 = Hand { cards: vec![card.clone()] };
        let hand2 = Hand { cards: vec![card] };
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
                });
            }
        }
        Self { cards }
    }
}

#[derive(Clone)]
pub struct Card {
    pub suit: Suit,
    pub face: CardFace,
}

#[derive(Clone)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Clone)]
pub enum CardFace {
    Ace,
    Number(u8),
    Face(Face),
}

#[derive(Clone)]
pub enum Face {
    Jack,
    Queen,
    King,
}