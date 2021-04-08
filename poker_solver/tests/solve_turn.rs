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
        raise_sizes: [vec![], vec![], vec![], vec![1.0]],
        all_in_threshold: 0f64,
    };
    let solver = Solver::init(SolverOptions {
        initial_state,
        hand_ranges: [
            String::from("22+,AT+,KT+,QT+,JT+"),
            String::from("22+,A5+,K9+,QT+,JT+"),
        ],
        betting_abstraction,
        card_abstraction: vec![String::from("null"), String::from("ochs")],
    })?;
    for _ in 0..20 {
        let br_equities = run_local_br(&solver, 10_000);
        println!("lbr EV {:?}", br_equities);
        let equities = solver.run(500_000);
        // solver.discount(i);
        println!("solver EV {:?}", equities);
        println!("dEV: {}", equities.iter().sum::<f64>().abs());
        let exploitability =
            0.5 * ((br_equities[0] - equities[0]) + (br_equities[1] - equities[1]));
        println!("exploitability {}, ", exploitability);
    }
    Ok(())
}
