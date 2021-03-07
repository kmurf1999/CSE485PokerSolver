use rust_poker::equity_calculator::exact_equity;
use rust_poker::hand_range::{Combo, HandRange};
use rust_poker::read_write::VecIO;
use rust_poker::HandIndexer;
use rayon::prelude::*;
use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*, BufReader, Error, SeekFrom, Write};
use std::mem::size_of;
use mpi::traits::*;

use std::time::Instant;

/// Filename to save ehs table to
const EHS_TABLE_FILENAME: &str = "EHS.dat";

type Precision = f32;

/// Generates an Expected Hand Strength (EHS) table.
/// Table is used to aid the creation of state abstractions for each betting round
///
/// Using indicies obtained from rust_poker::HandIndexer object
/// Lookup the EHS of any hand
pub fn generate_ehs_table() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let rank = world.rank() as usize;
    let size = world.size() as usize;
    let root_process = world.process_at_rank(0);
    let is_root = rank == 0;

    let cards_per_round: [usize; 4] = [2, 5, 6, 7];

    let indexers = [
        HandIndexer::init(1, [2].to_vec()),
        HandIndexer::init(2, [2, 3].to_vec()),
        HandIndexer::init(2, [2, 4].to_vec()),
        HandIndexer::init(2, [2, 5].to_vec()),
    ];

    // Create new file, exit if file exists
    let mut file: Option<File> = None;
    if is_root {
        file = Some(OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(EHS_TABLE_FILENAME)
        .unwrap());
    }
   

    const ROUNDS: usize = 4;
    for round in 0..ROUNDS {
        // for timing
        let start_time = Instant::now();
        // round for use in indexer
        let i_round = if round == 0 { 0 } else { 1 };
        // Number of isomorphic hands this round
        let num_hands = indexers[round].size(i_round) as usize;

        let (total_size, batch_size) = crate::split_into_batches(num_hands, size);

        let all_indicies: Vec<usize> = (0..total_size).into_iter().collect();
        let mut batch_indicies: Vec<usize> = vec![0usize; batch_size];

        if is_root {
            root_process.scatter_into_root(&all_indicies[..], &mut batch_indicies[..]);
        } else {
            root_process.scatter_into(&mut batch_indicies[..]);
        }

        let mut equity_all: Vec<Precision> = Vec::new();
        let mut equity_batch: Vec<Precision> = vec![0.0; batch_size];
        if is_root {
            equity_all = vec![0.0; total_size];
        }

        equity_batch.par_iter_mut().zip(batch_indicies).for_each(|(eq, index)| {
            if index >= num_hands {
                return;
            }
            if is_root && index.trailing_zeros() >= 12 {
                print!(
                    "round {}: {:.3}% \r",
                    round,
                    index as Precision / batch_size as Precision
                );
                io::stdout().flush().unwrap();
            }
            let mut cards: Vec<u8> = vec![0; cards_per_round[round]];
            // get hand at index
            indexers[round].get_hand(
                i_round,
                index.try_into().unwrap(),
                cards.as_mut_slice(),
            );
            // get hole cards
            let combo = Combo(cards[0], cards[1], 100);
            // get board mask
            let mut board_mask = 0u64;
            for c in &cards[2..cards_per_round[round]] {
                board_mask |= 1u64 << c;
            }
            // create ranges
            let hand_ranges = HandRange::from_strings(
                [combo.to_string(), "random".to_string()].to_vec(),
            );
            // run sim
            *eq = exact_equity(&hand_ranges, board_mask, 1).unwrap()[0] as Precision;
        });

        if is_root {
            root_process.gather_into_root(&equity_batch[..], &mut equity_all[..]);
        } else {
            root_process.gather_into(&equity_batch[..]);
        }

        // write round to file
        match &mut file {
            Some(f) => {
                f.write_slice_to_file(&equity_all[0..num_hands]).unwrap();
                // print duration
                let duration = start_time.elapsed().as_millis();
                println!(
                    "round {} done. took {}ms ({:.2} iterations / ms)",
                    round,
                    duration,
                    num_hands as Precision / duration as Precision
                );
            },
            None => {}
        }
    }
}

/// Structure to read f32 EHS values from the EHS.dat file
pub struct EhsReader {
    /// A hand indexer for each round of the game
    pub indexers: [HandIndexer; 4],
    offsets: [u64; 4],
    file: File,
}

impl EhsReader {
    /// Creates a new EHS reader
    pub fn new() -> Result<Self, Error> {
        let indexers = [
            HandIndexer::init(1, [2].to_vec()),
            HandIndexer::init(2, [2, 3].to_vec()),
            HandIndexer::init(2, [2, 4].to_vec()),
            HandIndexer::init(2, [2, 5].to_vec()),
        ];
        let mut offsets: [u64; 4] = [0; 4];
        for round in 1..4 {
            offsets[round] = offsets[round - 1] + indexers[round - 1].size(if round == 1 { 0 } else { 1 });
        }
        Ok(EhsReader {
            indexers,
            offsets,
            file: File::open(EHS_TABLE_FILENAME)?,
        })
    }

    /// Gets the EHS for a specific hand
    ///
    /// # Arguments
    ///
    /// * `cards` an array of 8 bit cards (first two indices are hole cards)
    /// * `round` round to evaluate (0 -> preflop, 4 -> river)
    pub fn get_ehs(&self, cards: &[u8], round: usize) -> Result<Precision, Error> {
        assert!(round < 5);
        let mut reader = BufReader::with_capacity(size_of::<Precision>(), &self.file);
        let index = self.indexers[round].get_index(cards);
        reader.seek(SeekFrom::Start(
            (index + self.offsets[round]) * size_of::<Precision>() as u64,
        ))?;
        let buffer = reader.fill_buf()?;
        let ehs = Precision::from_le_bytes(buffer.try_into().expect("slice length is incorrect"));
        Ok(ehs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    // test ehs::tests::bench_init_read          ... bench:     208,213 ns/iter (+/- 33,083)
    fn bench_init_reader(b: &mut Bencher) {
        b.iter(EhsReader::new);
    }

    #[bench]
    fn bench_get_ehs(b: &mut Bencher) {
        let reader = EhsReader::new().unwrap();
        let cards = [4u8 * 12, 4u8 * 12 + 1];
        b.iter(|| reader.get_ehs(&cards, 0));
    }

    #[test]
    fn test_preflop() {
        let reader = EhsReader::new().unwrap();
        let cards = [4u8 * 12, 4u8 * 12 + 1];
        let eq = reader.get_ehs(&cards, 0).unwrap();
        assert_eq!(eq, 0.85203713);
    }
}
