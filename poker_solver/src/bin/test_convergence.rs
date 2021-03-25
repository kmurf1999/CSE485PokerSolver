use poker_solver::solver::{Solver, SolverOptions};
use rust_poker::hand_range::get_card_mask;

fn test_river_convergence() {
    let options = SolverOptions {
        board_mask: get_card_mask("QhJdTsAc2c"),
        hand_ranges: vec!["random".to_string(), "random".to_string()],
        stacks: vec![1000, 1000],
        pot: 100,
        bet_sizes: vec![vec![vec![0.5]], vec![vec![0.5]]],
        raise_sizes: vec![vec![vec![1.0]], vec![vec![1.0]]],
        card_abstraction: vec!["ochs".to_string()],
    };
    let mut solver = Solver::init(options).unwrap();
    for _ in 0..50 {
        let mut evs = solver.run(100000);
        for ev in &mut evs {
            *ev /= 100000.0;
        }
        solver.save_regrets().unwrap();
        solver.save_strategy().unwrap();
        let mut new_ev = solver.run_br(10000000, 0);
        solver.load_regrets().unwrap();
        solver.load_strategy().unwrap();
        new_ev /= 10000000.0;
        // println!("start ev: {}, after ev: {}", evs[0], new_ev);
        println!("{:?},", new_ev - evs[0]);
    }
}

fn test_turn_convergence() {
    let options = SolverOptions {
        board_mask: get_card_mask("QhJdTsAc"),
        hand_ranges: vec!["random".to_string(), "random".to_string()],
        stacks: vec![1000, 1000],
        pot: 100,
        bet_sizes: vec![vec![vec![0.5], vec![0.5]], vec![vec![0.5], vec![0.5]]],
        raise_sizes: vec![vec![vec![1.0], vec![1.0]], vec![vec![1.0], vec![1.0]]],
        card_abstraction: vec!["null".to_string(), "ochs".to_string()],
    };
    let mut solver = Solver::init(options).unwrap();
    for i in 1..50 {
        let mut evs = solver.run(1000000);
        for ev in &mut evs {
            *ev /= 1000000.0;
        }
        solver.save_regrets().unwrap();
        solver.save_strategy().unwrap();
        let mut new_ev = solver.run_br(10000000, 0);
        solver.load_regrets().unwrap();
        solver.load_strategy().unwrap();
        new_ev /= 10000000.0;
        // println!("start ev: {}, after ev: {}", evs[0], new_ev);
        println!("{:?},", new_ev - evs[0]);
    }
}

fn test_flop_convergence() {
    let options = SolverOptions {
        board_mask: get_card_mask("QhJdTs"),
        hand_ranges: vec![
            "22+,AT+,KT+,QT+,JT+".to_string(),
            "22+,AT+,KT+,QT+,JT+".to_string(),
        ],
        stacks: vec![1000, 1000],
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
    let mut solver = Solver::init(options).unwrap();
    for i in 1..50 {
        let mut evs = solver.run(100_000_000);
        for ev in &mut evs {
            *ev /= 100_000_000.0;
        }
        solver.save_regrets().unwrap();
        solver.save_strategy().unwrap();
        let mut new_ev = solver.run_br(100_000_000, if i % 2 == 0 { 0 } else { 1 });
        // solver.discount((i as f64 / (i as f64 + 1.0)));
        solver.load_regrets().unwrap();
        solver.load_strategy().unwrap();
        new_ev /= 100_000_000.0;
        // println!("start ev: {}, after ev: {}", evs[0], new_ev);
        println!("{:?},", new_ev - evs[if i % 2 == 0 { 0 } else { 1 }]);
    }
}

fn main() {
    test_flop_convergence();
}
