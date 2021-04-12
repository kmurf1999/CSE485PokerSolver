#![feature(test)]

extern crate test;
use poker_solver::best_response::run_local_br;
use poker_solver::betting_abstraction::BettingAbstraction;
use poker_solver::solver::{Solver, SolverOptions};
use poker_solver::state::{GameState, GameStateOptions};

use std::error::Error;
use std::result::Result;
#[test]
fn test_solve_turn() -> Result<(), Box<dyn Error>> {
    // basic river solver
    let initial_state = GameState::new(GameStateOptions {
        stacks: [10000, 10000],
        initial_board: [0, 12, 14, 50, 52],
        blinds: [10, 5],
        pot: 1000,
    })?;
    let betting_abstraction = BettingAbstraction {
        bet_sizes: [vec![], vec![], vec![1.0], vec![1.0]],
        raise_sizes: [vec![], vec![], vec![1.0], vec![1.0]],
        all_in_threshold: 0f64,
    };
    let solver = Solver::init(SolverOptions {
        initial_state,
        hand_ranges: [String::from("random"), String::from("random")],
        betting_abstraction,
        card_abstraction: vec![String::from("null"), String::from("null")],
    })?;
    for i in 1..50 {
        let lbr_evs = run_local_br(&solver, 5000);
        println!("best response EV {:?}", lbr_evs);
        let evs = solver.run(0);
        solver.discount(i);
        println!("solver EV {:?}", evs);
        let exploitability = 0.5 * ((lbr_evs[0] - evs[0]) + (lbr_evs[1] - evs[1]));
        println!("exploitability: {}, ", exploitability);
    }
    Ok(())
}
