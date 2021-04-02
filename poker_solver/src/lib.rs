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
pub mod betting_abstraction;
/// Scores and prints card using an 8bit representation
pub mod card;
/// Loads card abstraction files into memory
pub mod card_abstraction;
/// Defines how to represent poker messages in json
// pub mod codec;
pub mod constants;
pub mod infoset;
/// Represents a betting round in poker
pub mod round;
/// postflop solver
pub mod solver;
/// Structures and methods for dealing with game state in texas holdem
pub mod state;

pub mod combos;

pub fn normalize(pdf: &mut Vec<f64>) {
    let sum: f64 = pdf.iter().sum();
    for prob in pdf.iter_mut() {
        *prob /= sum;
    }
}
