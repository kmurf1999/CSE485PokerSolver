use crate::ehs::EhsReader;
use itertools::Itertools;
use ndarray::prelude::*;
use ndarray::{Array2, Axis};
use rust_poker::read_write::VecIO;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::time::Instant;
use mpi::traits::*;


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
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let rank = world.rank() as usize;
    let size = world.size() as usize;
    let root_process = world.process_at_rank(0);
    let is_root = rank == 0;

    let start_time = Instant::now();

    println!("Generating histograms for round: {}, dim: {}", round, dim);

    let cards_per_round = [2, 5, 6, 7];
    let ehs_reader = EhsReader::new().unwrap();
    let round_size = ehs_reader.indexers[round].size(if round == 0 { 0 } else { 1 }) as usize;
    let size_per_thread = round_size / num_cpus::get();

    // dataset to generate
    let mut dataset: Option<Array2<f32>> = None;
    if is_root {
        dataset = Some(Array2::<f32>::zeros((round_size, dim)));
    }
    
    // crossbeam::scope(|scope| {
    //     for (i, mut slice) in dataset
    //         .axis_chunks_iter_mut(Axis(0), size_per_thread)
    //         .enumerate()
    //     {
    //         let ehs_reader = EhsReader::new().unwrap();
    //         let mut cards = vec![52u8; 7];
    //         scope.spawn(move |_| {
    //             for j in 0..slice.len_of(Axis(0)) {
    //                 let mut hist = slice.slice_mut(s![j, ..]);
    //                 if (i == 0) && j.trailing_zeros() >= 12 {
    //                     print!("{:.3}% \r", (100 * j) as f32 / size_per_thread as f32);
    //                     io::stdout().flush().unwrap();
    //                 }
    //                 let index = ((i * size_per_thread) + j) as u64;
    //                 ehs_reader.indexers[round].get_hand(
    //                     if round == 0 { 0 } else { 1 },
    //                     index,
    //                     cards.as_mut_slice(),
    //                 );
    //                 // build card mask for rejection sampling
    //                 let mut hand_mask: u64 = 0;
    //                 for c in &cards[0..cards_per_round[round]] {
    //                     hand_mask |= 1u64 << c;
    //                 }
    //                 // iterate over all posible remaining card combinations
    //                 let mut count = 0f32;
    //                 (0..52)
    //                     .combinations(7 - cards_per_round[round])
    //                     .for_each(|combo| {
    //                         let mut combo_mask = 0u64;
    //                         for c in &combo {
    //                             combo_mask |= 1u64 << c;
    //                         }
    //                         if (combo_mask & hand_mask) != 0 {
    //                             return;
    //                         }
    //                         for k in 0..combo.len() {
    //                             cards[cards_per_round[round] + k] = combo[k];
    //                         }
    //                         // get ehs on final round
    //                         let ehs = ehs_reader.get_ehs(&cards, 3).unwrap();
    //                         hist[get_bin(ehs, dim)] += 1.0;
    //                         count += 1.0;
    //                     });
    //                 hist /= count;
    //             }
    //         });
    //     }
    // })
    // .unwrap();

    // let duration = start_time.elapsed().as_millis();
    // println!("done. took {}ms", duration);
    Ok(dataset.unwrap())
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
            generate_ehs_histograms(0, 50).unwrap();
        })
    }
}
