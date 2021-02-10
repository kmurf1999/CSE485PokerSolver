use ndarray::prelude::*;
use crate::ehs::EHSReader;
use rand::distributions::Uniform;
use rand::rngs::SmallRng;
use std::time::Instant;
use ndarray::{Array2, Axis};
use std::fs::OpenOptions;
use rust_poker::read_write::VecIO;
use rand::{thread_rng, Rng, SeedableRng};
use std::io::{self, Write};
use std::error::Error;
use std::fs::File;


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
    return 0;
}

/// Generates Expected Hand Strength (EHS) histograms
/// 
/// # Arguments
/// * `n_threads` number of cpus threads to use
/// * `round` round to generate histograms for (0 -> preflop, 4 -> river)
/// * `dim` number of buckets per histogram
/// * `n_samples` number of samples per histogram
pub fn generate_ehs_histograms(n_threads: usize, round: usize, dim: usize, n_samples: usize) -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();

    println!(
        "Generating histograms for round: {}, n_samples: {}, dim: {}",
        round, n_samples, dim
    );

    // create file
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(format!("hist-r{}-d{}-s{}.dat", round, dim, n_samples))
        .unwrap();
    // Setup
    let cards_per_round = [2, 5, 6, 7];
    let mut thread_rng = thread_rng();
    let card_dist: Uniform<u8> = Uniform::from(0..52); // for faster sampling
    let ehs_reader = EHSReader::new().unwrap();
    let round_size = ehs_reader.indexers[round].size(if round > 0 { 1 } else { 0 }) as usize;
    let size_per_thread = round_size / n_threads;
    // dataset to generate
    let mut dataset = Array2::<f32>::zeros((round_size, dim));
    crossbeam::scope(|scope| {
        for (i, mut slice) in dataset
            .axis_chunks_iter_mut(Axis(0), size_per_thread)
            .enumerate()
        {
            let ehs_reader = EHSReader::new().unwrap();
            let mut rng = SmallRng::from_rng(&mut thread_rng).unwrap();
            let mut cards = vec![52u8; 7];
            scope.spawn(move |_| {
                for j in 0..slice.len_of(Axis(0)) {
                    let mut hist = slice.slice_mut(s![j, ..]);
                    if (i == 0) && (j & 0xff == 0) {
                        print!("{:.3}% \r", (100 * j) as f32 / size_per_thread as f32);
                        io::stdout().flush().unwrap();
                    }
                    let index = ((i * size_per_thread) + j) as u64;
                    ehs_reader.indexers[round].get_hand(
                        if round == 0 { 0 } else { 1 },
                        index,
                        cards.as_mut_slice(),
                    );
                    // build card mask for rejection sampling
                    let mut card_mask: u64 = 0;
                    for k in 0..cards_per_round[round] {
                        card_mask |= 1u64 << cards[k];
                    }
                    // create histogram
                    for _ in 0..n_samples {
                        // fill remaining board cards
                        let mut c_mask = card_mask;
                        for k in cards_per_round[round]..7 {
                            loop {
                                cards[k] = rng.sample(card_dist);
                                if (c_mask & 1u64 << cards[k]) == 0 {
                                    c_mask |= 1u64 << cards[k];
                                    break;
                                }
                            }
                        }
                        // get ehs and add to histogram
                        let ehs = ehs_reader.get_ehs(cards.as_slice(), 3).unwrap();

                        hist[get_bin(ehs, dim)] += 1f32;
                    }
                    // normalize histogram
                    hist /= n_samples as f32;
                }
            });
        }
    })
    .unwrap();

    file.write_slice_to_file(&dataset.as_slice().unwrap())?;
    let duration = start_time.elapsed().as_millis();
    println!("done. took {}ms", duration);
    Ok(())
}

/// Reads histogram data from file and returns a 2D array
///
/// # Arguments
/// * `round` the round of data to be read (0 is preflop, 4 is river)
/// * `dim` the dimension of the historgram (number of bins)
/// * `n_samples` the number of samples per histogram
pub fn read_ehs_histograms(
    round: usize,
    dim: usize,
    n_samples: usize,
) -> Result<Array2<f32>, Box<dyn Error>> {
    let mut file = File::open(format!("hist-r{}-d{}-s{}.dat", round, dim, n_samples))?;
    let flat_data = file.read_vec_from_file::<f32>()?;
    // TODO handle shape error instead
    let data = Array2::from_shape_vec((flat_data.len() / dim, dim), flat_data)?;
    Ok(data)
}