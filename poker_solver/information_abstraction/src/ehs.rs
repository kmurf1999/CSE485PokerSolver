use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::io::SeekFrom;
use std::mem::size_of;
use std::fs::OpenOptions;
use std::io;
use std::io::Write; // <--- bring flush() into scope
use std::time::Instant;

use rust_poker::equity_calculator::calc_equity;
use rust_poker::hand_range::{Combo, HandRange};
use rust_poker::read_write::VecIO;
use rust_poker::HandIndexer;

/// Filename to save ehs table to
const EHS_TABLE_FILENAME: &str = "EHS.dat";

type Precision = f32;

/// Generates an Expected Hand Strength (EHS) table.
/// Table is used to aid the creation of state abstractions for each betting round
///
/// Using indicies obtained from rust_poker::HandIndexer object
/// Lookup the EHS of any hand
pub fn generate_ehs_table(n_threads: usize) {
    let cards_per_round: [usize; 4] = [2, 5, 6, 7];

    let indexers = [
        HandIndexer::init(1, [2].to_vec()),
        HandIndexer::init(2, [2, 3].to_vec()),
        HandIndexer::init(2, [2, 4].to_vec()),
        HandIndexer::init(2, [2, 5].to_vec()),
    ];

    // Create new file, exit if file exists
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(EHS_TABLE_FILENAME)
        .unwrap();

    const ROUNDS: usize = 4;
    for i in 0..ROUNDS {
        let start_time = Instant::now();
        let round = if i == 0 { 0 } else { 1 };
        // Number of isomorphic hands this round
        let num_hands = indexers[i].size(round) as usize;
        let size_per_thread = num_hands / n_threads;
        let mut equity_table: Vec<Precision> = vec![0.0; num_hands];
        // spawn threads
        crossbeam::scope(|scope| {
            for (j, slice) in equity_table.chunks_mut(size_per_thread).enumerate() {
                scope.spawn(move |_| {
                    // setup thread variables
                    let mut board_mask: u64;
                    let mut combo: Combo;
                    let mut hand_ranges: Vec<HandRange>;
                    let mut cards: Vec<u8> = vec![0; cards_per_round[i]];
                    for k in 0..slice.len() {
                        // print progress to console every so often
                        if (j == 0) && (k & 0xfff == 0) {
                            print!(
                                "round {}: {:.3}% \r",
                                i,
                                (100 * k) as Precision / num_hands as Precision
                            );
                            io::stdout().flush().unwrap();
                        }

                        // get hand at index
                        indexers[i].get_hand(
                            round,
                            (j * size_per_thread + k) as u64,
                            cards.as_mut_slice(),
                        );
                        // get hole cards
                        combo = Combo(cards[0], cards[1], 100);
                        // get board mask
                        board_mask = 0;
                        for n in 2..cards_per_round[i] {
                            board_mask |= 1u64 << cards[n];
                        }
                        // create ranges
                        hand_ranges = HandRange::from_strings(
                            [combo.to_string(), "random".to_string()].to_vec(),
                        );
                        // run sim
                        slice[k] = match i {
                            // preflop error is around 0.0-0.4%
                            0 => calc_equity(&hand_ranges, board_mask, 4, 100000)[0],
                            1 => calc_equity(&hand_ranges, board_mask, 2, 10000)[0],
                            2 => calc_equity(&hand_ranges, board_mask, 1, 5000)[0],
                            3 => calc_equity(&hand_ranges, board_mask, 1, 5000)[0],
                            _ => panic!("Invalid round"),
                        } as Precision;
                    }
                });
            }
        })
        .unwrap();

        // write round to file
        file.write_slice_to_file(&equity_table.as_slice()).unwrap();
        // print duration
        let duration = start_time.elapsed().as_millis();
        println!(
            "round {} done. took {}ms ({:.2} iterations / ms)",
            i,
            duration,
            num_hands as Precision / duration as Precision
        );
    }
}

/// Structure to read f32 EHS values from the EHS.dat file
pub struct EHSReader {
    /// A hand indexer for each round of the game
    pub indexers: [HandIndexer; 4],
    offsets: [u64; 4],
    file: File,
}

impl EHSReader {
    /// Creates a new EHS reader
    pub fn new() -> Result<Self, Error> {
        let indexers = [
            HandIndexer::init(1, [2].to_vec()),
            HandIndexer::init(2, [2, 3].to_vec()),
            HandIndexer::init(2, [2, 4].to_vec()),
            HandIndexer::init(2, [2, 5].to_vec()),
        ];
        let mut offsets: [u64; 4] = [0; 4];
        for i in 1..4 {
            offsets[i] = offsets[i - 1] + indexers[i - 1].size(if i == 1 { 0 } else { 1 });
        }
        Ok(EHSReader {
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
