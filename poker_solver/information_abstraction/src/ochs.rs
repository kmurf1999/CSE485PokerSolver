use ndarray::parallel::prelude::*;
use ndarray::prelude::*;
use rust_poker::equity_calculator::exact_equity;
use rust_poker::hand_range::{get_card_mask, HandRange};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;
use mpi::traits::*;
use std::fs::{OpenOptions, File};
use std::result::Result;
use rust_poker::read_write::VecIO;

use std::error::Error;


use rust_poker::{
    constants::{RANK_TO_CHAR, SUIT_TO_CHAR},
    HandIndexer,
};
use std::io::{self, Write};

static OCHS_CLUSTERS: &[&str; 8] = &[
    "88+",
    "A9o,ATo,AJo,AQo,AKo,KTo,KJo,KQo,A7s,A8s,A9s,ATs,AJs,AQs,AKs,K9s,KTs,KJs,KQs,QTs,QJs,66,77",
    "A2o,A3o,A4o,A5o,A6o,A7o,A8o,K5o,K6o,K7o,K8o,K9o,33,44,55,A2s,A3s,A4s,A5s,A6s,K3s,K4s,K5s,K6s,K7s,K8s",
    "Q8o,Q9o,QTo,QJo,J8o,J9o,JTo,T9o,Q6s,Q7s,Q8s,Q9s,J7s,J8s,J9s,JTs,T7s,T8s,T9s",
    "22,K2o,K3o,K4o,Q2o,Q3o,Q4o,Q5o,Q6o,Q7o,J4o,J5o,J6o,J7o,K2s,Q2s,Q3s,Q4s,Q5s,J2,J3s,J4s,J5s,J6s",
    "T3s,T4s,T5s,T6s,95s,96s,97s,98s,85s,86s,75s,76s,65s,76o,86o,96o,T6o,87o,97o,T7o,98o,T8o",
    "T2s,92s,93s,94s,82s,83s,84s,74s,75o,85o,95o,T5o,84o,94o,T4o,93o,T3o,J3o,92o,T2o,J2o",
    "32s,42s,52s,62s,72s,43s,53s,63s,73s,54s,64s,32o,42o,43o,52o,53o,54o,62o,63o,64o,65o,72o,73o,74o,82o,83o"
];

#[inline(always)]
fn cards_to_str(cards: &[u8]) -> String {
    let mut out = String::new();
    for i in 0..cards.len() {
        let rank = cards[i] / 4;
        let suit = cards[i] % 4;
        out.push(RANK_TO_CHAR[usize::from(rank)]);
        out.push(SUIT_TO_CHAR[usize::from(suit)]);
    }
    out
}

pub fn gen_ochs_features(round: u8) -> Result<(), Box<dyn Error>> {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let rank = world.rank() as usize;
    let size = world.size() as usize;
    let root_process = world.process_at_rank(0);
    let is_root = rank == 0;

    let start_time = Instant::now();

    if is_root {
        println!("Generating ochs vectors for round: {}", 3);
    }

    // Create new file, exit if file exists
    let mut file: Option<File> = None;
    if is_root {
        // create file
        file = Some(OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(format!(
                "ochs-features-r{}.dat",
                round
            ))?);

    }

    let indexer = match round {
        1 => HandIndexer::init(2, [2, 3].to_vec()),
        2 => HandIndexer::init(2, [2, 4].to_vec()),
        3 => HandIndexer::init(2, [2, 5].to_vec()),
        _ => panic!("invalid round"),
    };

    let n_board_cards = match round {
        1 => 3,
        2 => 4,
        3 => 5,
        _ => panic!("invalid round"),
    };

    let round_size = if round == 0 {
        indexer.size(0)
    } else {
        indexer.size(1)
    } as usize;

    let (total_size, batch_size) = crate::split_into_batches(round_size, size);
    let all_indicies: Vec<usize> = (0..total_size).into_iter().collect();
    let mut batch_indicies: Vec<usize> = vec![0usize; batch_size];

    if is_root {
        root_process.scatter_into_root(&all_indicies[..], &mut batch_indicies[..]);
    } else {
        root_process.scatter_into(&mut batch_indicies[..]);
    }


    let mut ochs_all: Array2<f32> = Array2::zeros((0, 0));
    let mut ochs_batch: Array2<f32> = Array2::zeros((batch_size, OCHS_CLUSTERS.len()));
    if is_root {
        ochs_all = Array2::zeros((total_size, OCHS_CLUSTERS.len()));
    }

    // 123156254 * 32 bits * 8 = 3.941000128 gigabytes
    let counter = AtomicU32::new(0);

    ochs_batch
        .axis_iter_mut(Axis(0))
        .into_par_iter()
        .enumerate()
        .for_each(|(i, mut ochs_vec)| {
            let indicies = &batch_indicies;
            let hand_index = indicies[i] as u64;
            // print progress
            let c = counter.fetch_add(1, Ordering::SeqCst);
            if is_root && i.trailing_zeros() >= 12 {
                print!("{:.3}% \r", c as f32 * 100.0 / batch_size as f32);
                io::stdout().flush().unwrap();
            }
            let mut cards = [0u8; 7];
            indexer.get_hand(1, hand_index, &mut cards);
            let hole_cards = cards_to_str(&cards[0..2]);
            let board_cards = cards_to_str(&cards[2..(n_board_cards + 2)]);
            let board_mask = get_card_mask(board_cards.as_str());
            for j in 0..OCHS_CLUSTERS.len() {
                let ranges = HandRange::from_strings(
                    [hole_cards.to_string(), OCHS_CLUSTERS[j].to_string()].to_vec(),
                );
                let equity: f32 = match exact_equity(&ranges, board_mask, 1) {
                    Ok(eq) => eq[0] as f32,
                    Err(_) => 0.5
                };
                ochs_vec[j] = equity;
            }
        });

    
    if is_root {
        root_process.gather_into_root(&ochs_batch.as_slice().unwrap()[..], &mut ochs_all.as_slice_mut().unwrap()[..]);
    } else {
        root_process.gather_into(&ochs_batch.as_slice().unwrap()[..]);
    }

    match &mut file {
        Some(f) => {
            let duration = start_time.elapsed().as_millis();
            println!("done. took {}ms", duration);
            f.write_slice_to_file(&ochs_batch.as_slice().unwrap()[0..(round_size * OCHS_CLUSTERS.len())])?;
        },
        None => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_poker::HandIndexer;

    #[test]
    fn test_get_hole_cards() {
        let indexer = HandIndexer::init(2, [2, 5].to_vec());
        let mut cards = [0u8; 7];
        indexer.get_hand(1, 5000, &mut cards);
        assert_eq!(cards_to_str(&cards[0..2]), "AsTh");
        assert_eq!(cards_to_str(&cards[2..7]), "6d8d9d2c3c");
    }
}
