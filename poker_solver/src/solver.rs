use crate::card_abstraction::{CardAbstraction, CardAbstractionOptions};
use crate::constants::*;
use crate::game_node::GameNode;
use crate::round::BettingRound;
use crate::tree::{Node, Tree};
use crate::tree_builder::{TreeBuilder, TreeBuilderOptions};
use rust_poker::hand_range::{get_card_mask, HandRange};
use std::convert::TryFrom;
use std::result::Result;
use thiserror::Error as ThisError;

type Error = Box<dyn std::error::Error>;

#[derive(Debug, ThisError)]
pub enum SolverError {
    #[error("too many players")]
    TooManyPlayers,
    #[error("not enough players")]
    TooFewPlayers,
    #[error("invalid board mask")]
    InvalidBoard,
    #[error("array sizes don't match")]
    PlayerCountMismatch,
    #[error("invalid number of rounds in bet sizes")]
    InvalidBetSizes,
    #[error("invalid number of rounds in raise sizes")]
    InvalidRaiseSizes,
    #[error("pot must be greater than zero")]
    InvalidPotSize,
    #[error("invalid abstraction")]
    InvalidCardAbstraction,
}

/// options for running a postflop solver simulation
#[derive(Debug)]
pub struct SolverOptions {
    /// initial board mask
    pub board_mask: u64,
    /// hand range for each player
    pub hand_ranges: Vec<HandRange>,
    /// initial stack sizes
    pub stacks: Vec<u32>,
    /// initial pot size
    pub pot: u32,
    /// array of bet sizes for each player for each round
    pub bet_sizes: Vec<Vec<Vec<f64>>>,
    /// array of raise sizes for each player for each round
    pub raise_sizes: Vec<Vec<Vec<f64>>>,
    /// an abstraction for every round
    pub card_abstraction: Vec<String>,
}

#[derive(Debug)]
pub struct Solver {
    /// game tree including all chance, private, and action nodes
    game_tree: Tree<GameNode>,
    /// initial board as 64 bit mask
    initial_board: u64,
    /// a card abstraction for every round
    card_abstraction: Vec<CardAbstraction>,
}

impl Solver {
    pub fn init(options: SolverOptions) -> Result<Solver, Error> {
        // check if initial board is valid
        // must be between 3 and 5 cards
        let num_board_cards = options.board_mask.count_ones();
        if options.board_mask >= (1u64 << CARD_COUNT) {
            return Err(SolverError::InvalidBoard.into());
        }
        // number of betting rounds in this tree
        let num_rounds = match num_board_cards {
            3 => 3,
            4 => 2,
            5 => 1,
            _ => {
                return Err(SolverError::InvalidBoard.into());
            }
        };
        let round = match num_board_cards {
            3 => BettingRound::FLOP,
            4 => BettingRound::TURN,
            5 => BettingRound::RIVER,
            _ => {
                return Err(SolverError::InvalidBoard.into());
            }
        };

        // check if player count is valid
        let n_players = options.hand_ranges.len();
        if n_players < MIN_PLAYERS {
            return Err(SolverError::TooFewPlayers.into());
        }
        if n_players > MAX_PLAYERS {
            return Err(SolverError::TooManyPlayers.into());
        }
        // check if all argument sizes match
        if options.stacks.len() != n_players {
            return Err(SolverError::PlayerCountMismatch.into());
        }
        if options.bet_sizes.len() != n_players {
            return Err(SolverError::PlayerCountMismatch.into());
        }
        if options.raise_sizes.len() != n_players {
            return Err(SolverError::PlayerCountMismatch.into());
        }
        if options.pot == 0 {
            return Err(SolverError::InvalidPotSize.into());
        }
        // check bet sizes
        for player_bet_sizes in &options.bet_sizes {
            if player_bet_sizes.len() != num_rounds {
                return Err(SolverError::InvalidBetSizes.into());
            }
            for round_bet_sizes in player_bet_sizes {
                for bet_size in round_bet_sizes {
                    if *bet_size < 0.0 {
                        return Err(SolverError::InvalidBetSizes.into());
                    }
                }
            }
        }
        // check raise sizes
        for player_raise_sizes in &options.raise_sizes {
            if player_raise_sizes.len() != num_rounds {
                return Err(SolverError::InvalidRaiseSizes.into());
            }
            for round_raise_sizes in player_raise_sizes {
                for raise_size in round_raise_sizes {
                    if *raise_size < 0.0 {
                        return Err(SolverError::InvalidBetSizes.into());
                    }
                }
            }
        }
        if options.card_abstraction.len() != num_rounds {
            return Err(SolverError::InvalidCardAbstraction.into());
        }

        let card_abstraction: Result<Vec<CardAbstraction>, Error> = options
            .card_abstraction
            .iter()
            .enumerate()
            .map(|(i, abs_string)| {
                let r = BettingRound::try_from(usize::from(round) + i)?;
                let ca_opts = CardAbstractionOptions {
                    k: 5000,
                    d: if r == BettingRound::RIVER { 8 } else { 50 },
                    round: r,
                    abs_type: abs_string.to_string(),
                };
                CardAbstraction::load(ca_opts)
            })
            .collect();

        let card_abstraction = match card_abstraction {
            Ok(ca) => ca,
            Err(e) => {
                return Err(e);
            }
        };

        let tree_options = TreeBuilderOptions {
            blinds: None,
            stacks: options.stacks,
            pot: options.pot,
            round,
            bet_sizes: options.bet_sizes,
            raise_sizes: options.raise_sizes,
        };
        let game_tree = TreeBuilder::build(&tree_options)?;

        let solver = Solver {
            game_tree,
            initial_board: options.board_mask,
            card_abstraction,
        };
        Ok(solver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_valid() {
        let options = SolverOptions {
            board_mask: get_card_mask("AhAdAc"),
            hand_ranges: HandRange::from_strings(vec!["random".to_string(), "random".to_string()]),
            stacks: vec![10000, 10000],
            pot: 100,
            bet_sizes: vec![
                vec![vec![0.5], vec![0.5], vec![0.5]],
                vec![vec![0.5], vec![0.5], vec![0.5]],
            ],
            raise_sizes: vec![
                vec![vec![1.0], vec![1.0], vec![1.0]],
                vec![vec![1.0], vec![1.0], vec![1.0]],
            ],
            card_abstraction: vec!["null".to_string(), "emd".to_string(), "ochs".to_string()],
        };
        let solver = Solver::init(options);
        assert_eq!(solver.is_ok(), true);
    }
}
