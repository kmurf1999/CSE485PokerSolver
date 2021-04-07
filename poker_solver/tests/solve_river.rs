#![feature(test)]

extern crate test;
use poker_solver::best_response::run_local_br;
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
        initial_board: [0, 12, 14, 50, 6],
        blinds: [10, 5],
        pot: 1000,
    })?;
    let betting_abstraction = BettingAbstraction {
        bet_sizes: [vec![], vec![], vec![], vec![1.0]],
        raise_sizes: [vec![], vec![], vec![], vec![1.0]],
        all_in_threshold: 0f64,
    };
    let solver = Solver::init(SolverOptions {
        initial_state,
        hand_ranges: [String::from("random"), String::from("random")],
        betting_abstraction,
        card_abstraction: vec![String::from("null")],
    })?;
    for _ in 0..20 {
        let br_equities = run_local_br(&solver, 500);
        println!("best response EV {:?}", br_equities);
        let equities = solver.run(100_000);
        // solver.discount(i);
        println!("solver EV {:?}", equities);
        let exploitability =
            0.5 * ((br_equities[0] - equities[0]) + (br_equities[1] - equities[1]));
        println!("exploitability: {}, ", exploitability);
    }

    // let equities = solver.run(100000);
    // println!("{:?}", equities);
    // let br_eq = solver.run_br(1000, 0);
    // println!("{}", br_eq);

    Ok(())
}
