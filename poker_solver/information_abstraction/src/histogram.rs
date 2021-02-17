use crate::ehs::EHSReader;
use itertools::Itertools;
use ndarray::parallel::prelude::*;
use ndarray::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

use ndarray::{Array1, Array2, Axis};
use rust_poker::read_write::VecIO;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::time::Instant;

#[inline(always)]
fn get_bin(value: f32, bins: usize) -> usize {
    let interval = 1f32 / bins as f32;
    let mut bin = bins - 1;
    let mut threshold = 1f32 - interval;
    while bin > 0 {
        if value > threshold {
            return bin;
        }
        bin -= 1;
        threshold -= interval;
    }
    0
}

/// Generates Expected Hand Strength (EHS) histograms
///
/// # Arguments
/// * `round` round to generate histograms for (0 -> preflop, 4 -> river)
/// * `dim` number of buckets per histogram
pub fn generate_ehs_histograms(round: usize, dim: usize) -> Result<Array2<f32>, Box<dyn Error>> {
    let start_time = Instant::now();

    println!("Generating histograms for round: {}, dim: {}", round, dim);

    // Setup
    let cards_per_round = [2, 5, 6, 7];
    let ehs_reader = EHSReader::new().unwrap();
    let round_size = ehs_reader.indexers[round].size(if round == 0 { 0 } else { 1 }) as usize;
    let counter = AtomicU32::new(0);

    // dataset to generate
    let mut dataset = Array2::<f32>::zeros((round_size, dim));
    dataset
        .axis_iter_mut(Axis(0))
        .into_par_iter()
        .enumerate()
        .for_each(|(i, mut hist)| {
            if i > 5 {
                return;
            }
            let ehs_reader = EHSReader::new().unwrap();
            let c = counter.fetch_add(1, Ordering::SeqCst);
            if round == 0 || i.trailing_zeros() >= 12 {
                print!("{}/{}\r", c, round_size);
                io::stdout().flush().unwrap();
            }

            let mut cards = vec![52u8; 7];
            // get initial cards
            ehs_reader.indexers[round].get_hand(
                if round == 0 { 0 } else { 1 },
                i as u64,
                &mut cards,
            );
            // generate hand mask for sampling
            let mut hand_mask = 0u64;
            for c in &cards[0..cards_per_round[round]] {
                hand_mask |= 1u64 << c;
            }
            // iterate over all posible remaining card combinations
            let mut count = 0f32;
            (0..52)
                .combinations(7 - cards_per_round[round])
                .for_each(|combo| {
                    let mut combo_mask = 0u64;
                    for c in &combo {
                        combo_mask |= 1u64 << c;
                    }
                    if (combo_mask & hand_mask) != 0 {
                        return;
                    }
                    for j in 0..combo.len() {
                        cards[cards_per_round[round] + j] = combo[j];
                    }
                    // get ehs on final round
                    let ehs = ehs_reader.get_ehs(&cards, 3).unwrap();
                    hist[get_bin(ehs, dim)] += 1.0;
                    count += 1.0;
                });
            hist /= count;
        });

    let duration = start_time.elapsed().as_millis();
    println!("done. took {}ms", duration);
    Ok(dataset)
}
/// Reads histogram data from file and returns a 2D array
///
/// # Arguments
/// * `round` the round of data to be read (0 is preflop, 4 is river)
/// * `dim` the dimension of the historgram (number of bins)
/// * `n_samples` the number of samples per histogram
pub fn read_ehs_histograms(round: usize, dim: usize) -> Result<Array2<f32>, Box<dyn Error>> {
    let mut file = File::open(format!("hist-r{}-d{}.dat", round, dim))?;
    let flat_data = file.read_vec_from_file::<f32>()?;
    // TODO handle shape error instead
    let data = Array2::from_shape_vec((flat_data.len() / dim, dim), flat_data)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    // test histogram::tests::bench_gen_ehs_hist ... bench: 120,592,094 ns/iter (+/- 20,527,346)
    fn bench_gen_ehs_hist(b: &mut Bencher) {
        b.iter(|| {
            generate_ehs_histograms(0, 50);
        })
    }
}
