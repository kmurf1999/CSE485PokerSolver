//!
//! # Poker Solver
//!
//! Structures and methods for solving heads up no limit texas holdem situtations using counter-factual regret minimization
//!
#![feature(array_map)]
#![allow(soft_unstable)]
#![feature(test)]
extern crate test;

/// Player actions in texas holdem
pub mod action;
/// Local best response implementation
/// used to approximate exploitability of solutions
pub mod best_response;
/// Betting abstraction used by solver
pub mod betting_abstraction;
/// Scores and prints card using an 8bit representation
pub mod card;
/// Loads card abstraction files into memory
pub mod card_abstraction;
pub mod constants;
/// Stores regrets and strategy for each infoset
pub mod infoset;
/// Represents a betting round in poker
pub mod round;
/// postflop solver
pub mod solver;
/// Structures and methods for dealing with game state in texas holdem
pub mod state;

/// Normalizes a probability density function so that the sum is 1
pub fn normalize(pdf: &mut Vec<f64>) {
    let sum: f64 = pdf.iter().sum();
    if sum <= 0.0 {
        return;
    }
    for prob in pdf.iter_mut() {
        *prob /= sum;
    }
}
