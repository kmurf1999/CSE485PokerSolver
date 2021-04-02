#![feature(test)]

extern crate test;
use poker_solver::betting_abstraction::BettingAbstraction;
use poker_solver::solver::{Solver, SolverOptions};
use poker_solver::state::{GameState, GameStateOptions};
use std::error::Error;
use std::result::Result;
#[test]
fn test_solve_river() -> Result<(), Box<dyn Error>> {
    // basic river solver
    let initial_state = GameState::new(GameStateOptions {
        stacks: [10000, 10000],
        initial_board: [0, 1, 2, 3, 4],
        wagers: [0, 0],
        pot: 1000,
    })?;
    let betting_abstraction = BettingAbstraction {
        bet_sizes: [vec![], vec![], vec![], vec![0.5, 1.0]],
        raise_sizes: [vec![], vec![], vec![], vec![1.0]],
        all_in_threshold: 0f64,
    };
    let mut solver = Solver::init(SolverOptions {
        initial_state,
        hand_ranges: [String::from("random"), String::from("random")],
        betting_abstraction,
        card_abstraction: vec![String::from("null")],
    })?;
    let equities = solver.run(1000);
    println!("{:?}", equities);
    let br_eq = solver.run_br(1000, 0);
    println!("{}", br_eq);
    let equities = solver.run(1000);
    println!("{:?}", equities);
    let br_eq = solver.run_br(1000, 0);
    println!("{}", br_eq);

    Ok(())
}
