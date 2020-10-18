use std::iter::FromIterator;

use rust_poker::hand_evaluator::{evaluate, CARDS, Hand};
use rust_poker::constants::{ RANK_TO_CHAR, SUIT_TO_CHAR };


/// use 8 bit integer to represent a playing card
/// valid cards n: 0->51
/// where n is 4 * rank + suit
pub type Card = u8;

pub fn cards_to_str(cards: &[Card]) -> String {
    let mut chars: Vec<char> = Vec::new();
    cards.into_iter().filter(|c| **c < 52).for_each(|c| {
        chars.push(RANK_TO_CHAR[usize::from(*c >> 2)]);
        chars.push(SUIT_TO_CHAR[usize::from(*c & 3)]);
    });
    String::from_iter(chars)
}

/// Scores a texas holdem hand
/// 
/// Combines private cards and public board cards
/// to create the best 5-card hand possible
/// and returns its score
/// 
/// higher score is better
pub fn score_hand(board: &[Card], private_cards: &[Card]) -> u16 {
    let mut hand = Hand::empty();
    board.into_iter().for_each(|c| {
        hand += CARDS[usize::from(*c)];
    });
    private_cards.into_iter().for_each(|c| {
        hand += CARDS[usize::from(*c)];
    });
    return evaluate(&hand);
}

pub fn player_hand_score(private_cards: &[Card]) -> u16 {
    let mut hand = Hand::empty();
    private_cards.into_iter().for_each(|c| {
        hand += CARDS[usize::from(*c)];
    });
    return evaluate(&hand);
}

