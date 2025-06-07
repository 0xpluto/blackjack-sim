use crate::types::{Card, CardFace, Hand, PlayerChoice, PlayerChoices};


pub struct BasicStrategy;

impl BasicStrategy {

    pub fn choice(hand: &Hand, dealer_card: &Card, choices: PlayerChoices) -> PlayerChoice {
        let value = hand.value();
        let splitable_card = if choices.contains(PlayerChoices::SPLIT) {
            hand.splitable_card()
        } else {
            None
        };
        let dealer_value = dealer_card.face_value();
        let is_soft = hand.is_soft();

        use CardFace::*;
        match (dealer_value, value, is_soft, splitable_card) {

            // Splits

            (_, _, _, Some(Ace)) => {
                PlayerChoice::Split // Always split aces
            }
            (_, _, _, Some(Face(_))) => {
                PlayerChoice::Stand // Never split face cards
            }
            (_, _, _, Some(Number(10))) => {
                PlayerChoice::Stand // Never split 10s
            }
            (d, _, _, Some(Number(9))) => {
                if d == 7 || d >= 10 {
                    PlayerChoice::Stand // Stand on 9s against dealer 7, 10, or Ace
                } else {
                    PlayerChoice::Split // Split 9s against dealer 2-6, 8-9
                }
            }
            (d, _, _, Some(Number(8))) => {
                if d >= 10 {
                    PlayerChoice::Hit // Hit on 8s against dealer 10, or Ace
                } else {
                    PlayerChoice::Split // Split 8s against dealer 2-9
                }
            }
            (d, _, _, Some(Number(7))) => {
                if d >= 8 {
                    PlayerChoice::Hit // Stand on 7s against dealer 8, 9, 10, or Ace
                } else {
                    PlayerChoice::Split // Split 7s against dealer 2-6
                }
            }
            (d, _, _, Some(Number(6))) => {
                if d >= 7 {
                    PlayerChoice::Hit // Stand on 6s against dealer 7, 8, 9, 10, or Ace
                } else {
                    PlayerChoice::Split // Split 6s against dealer 2-6
                }
            }
            (d, _, _, Some(Number(5))) => {
                if d <= 9 {
                    PlayerChoice::Double // Double on 5s against dealer 2-9
                } else {
                    PlayerChoice::Hit // Hit on 5s against dealer 10 or Ace
                }
            }
            (d, _, _, Some(Number(4))) => {
                if d == 5 || d == 6 {
                    PlayerChoice::Split // Double on 4s against dealer 5-6
                } else {
                    PlayerChoice::Hit // Hit on 4s against dealer 2-4, 7-10, or Ace
                }
            }
            (d, _, _, Some(Number(x))) if x <= 3 => {
                if d >= 8 {
                    PlayerChoice::Hit // Hit 3s/2s against dealer 8-10, or Ace
                } else {
                    PlayerChoice::Split // Split 3s/2s against dealer 2-7
                }
            }
            // Unreachable splits
            (_, _, _, Some(Number(x))) if !(2..=10).contains(&x) => {
                unreachable!("Splitable card not in range 2-10: {}", x)
            }


            // Soft totals

            (_, p, true, _) if p >= 19 => {
                PlayerChoice::Stand // Stand on soft 19 or higher
            }
            (d, 18, true, _) => {
                if d >= 9 {
                    PlayerChoice::Hit
                } else if d == 7 || d == 8 || d == 2 {
                    PlayerChoice::Stand // Stand on soft 18 against dealer 7 or 8
                } else {
                    PlayerChoice::Double // Double on soft 18 against dealer 3-6
                }
            }
            (d, 17, true, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (3..=6).contains(&d) {
                    PlayerChoice::Double // Double on soft 17 against dealer 3-6
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            (d, p, true, _) if (15..=16).contains(&p) => {
                if choices.contains(PlayerChoices::DOUBLE) && (4..=6).contains(&d){
                    PlayerChoice::Double // Double on soft 16 against dealer 3-6
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            (d, p, true, _) if (13..=14).contains(&p) => {
                if choices.contains(PlayerChoices::DOUBLE) && (5..=6).contains(&d) {
                    PlayerChoice::Double // Double on soft 14 against dealer 4-6
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            // Unreachable soft totals
            (_, p, true, _) if p <= 12 => {
                unreachable!("Soft total below 13")
            }

            // Hard totals

            (_, 17..=u8::MAX, _, _) => {
                PlayerChoice::Stand // Stand on hard 17 or higher
            }
            (d, 13..=16, _, _) => {
                if d <= 6 {
                    PlayerChoice::Stand // Stand on hard 13-16 against dealer 2-6
                } else {
                    PlayerChoice::Hit // Hit on hard 13-16 against dealer 7-10, or Ace
                }
            }
            (d, 12, _, _) => {
                if (4..=6).contains(&d) {
                    PlayerChoice::Stand // Stand on hard 12 against dealer 4-6
                } else {
                    PlayerChoice::Hit // Hit on hard 12 against dealer 2-3, 7-10, or Ace
                }
            }
            (d, 11, _, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (2..=10).contains(&d) {
                    PlayerChoice::Double // Double on hard 11 against dealer 2-10
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            (d, 10, _, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (2..=9).contains(&d) {
                    PlayerChoice::Double // Double on hard 10 against dealer 2-9
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            (d, 9, _, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (3..=6).contains(&d) {
                    PlayerChoice::Double // Double on hard 9 against dealer 3-6
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            (_, 0_u8..=8_u8, _, _) => {
                PlayerChoice::Hit // Hit on hard 8 or lower
            }
        }
    }
}
