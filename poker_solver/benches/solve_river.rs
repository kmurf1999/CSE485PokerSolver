#![feature(test)]

extern crate test;
use poker_solver::best_response::run_local_br;
use poker_solver::betting_abstraction::BettingAbstraction;
use poker_solver::solver::{Solver, SolverOptions};
use poker_solver::state::{GameState, GameStateOptions};
use std::error::Error;
use std::result::Result;
use test::bench::Bencher;

#[bench]
fn bench_init(b: &mut Bencher) -> Result<(), Box<dyn Error>> {
    // 220,327,172 ns/iter (+/- 54,654,690)
    // basic river solver setup
    let initial_state = GameState::new(GameStateOptions {
        stacks: [10000, 10000],
        initial_board: [0, 1, 2, 3, 4],
        blinds: [10, 5],
        pot: 1000,
    })?;

    b.iter(|| {
        let betting_abstraction = BettingAbstraction {
            bet_sizes: [vec![], vec![], vec![], vec![0.5, 1.0]],
            raise_sizes: [vec![], vec![], vec![], vec![1.0]],
            all_in_threshold: 0f64,
        };
        let solver = Solver::init(SolverOptions {
            initial_state: initial_state.clone(),
            hand_ranges: [String::from("random"), String::from("random")],
            betting_abstraction,
            card_abstraction: vec![String::from("null")],
        });
        assert!(solver.is_ok());
    });
    Ok(())
}

#[bench]
fn bench_run_1(b: &mut Bencher) -> Result<(), Box<dyn Error>> {
    // 36,375 ns/iter (+/- 4,496)
    // basic river solver setup
    let initial_state = GameState::new(GameStateOptions {
        stacks: [10000, 10000],
        initial_board: [0, 1, 2, 3, 4],
        blinds: [10, 5],
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

    b.iter(|| {
        solver.run(1);
    });
    Ok(())
}

#[bench]
fn bench_run_10000(b: &mut Bencher) -> Result<(), Box<dyn Error>> {
    // 423,644,946 ns/iter (+/- 72,076,016)
    // basic river solver setup
    let initial_state = GameState::new(GameStateOptions {
        stacks: [10000, 10000],
        initial_board: [0, 1, 2, 3, 4],
        blinds: [10, 5],
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

    b.iter(|| {
        solver.run(10000);
    });
    Ok(())
}

#[bench]
fn bench_run_br_1(b: &mut Bencher) -> Result<(), Box<dyn Error>> {
    // 3,002,107 ns/iter (+/- 619,469)
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
    let mut solver = Solver::init(SolverOptions {
        initial_state,
        hand_ranges: [String::from("random"), String::from("random")],
        betting_abstraction,
        card_abstraction: vec![String::from("null")],
    })?;
    solver.run(100_000);
    b.iter(|| {
        run_local_br(&solver, 100);
    });
    Ok(())
}
