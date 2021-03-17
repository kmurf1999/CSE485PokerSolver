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
pub mod codec;
/// Used for iterating over card combinations
pub mod combos;
/// A tree node for representing a poker game tree
pub mod game_node;
/// Monte-carlo counter factual regret minimization implementation
pub mod mccfr;
/// Represents a betting round in poker
pub mod round;
/// Maps sparse arrays to dense to save memory
pub mod sparse_and_dense;
/// Structures and methods for dealing with game state in texas holdem
pub mod state;
/// A tree structure implemented in rust
pub mod tree;
/// Methods for building poker game trees
pub mod tree_builder;
