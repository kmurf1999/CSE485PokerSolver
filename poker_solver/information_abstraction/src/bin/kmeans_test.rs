use information_abstraction::distance;
use information_abstraction::histogram::read_ehs_histograms;
use information_abstraction::kmeans::{HammerlyKmeans, Kmeans, VanillaKmeans};
use ndarray::prelude::*;
use rust_poker::constants::{RANK_TO_CHAR, SUIT_TO_CHAR};
use rust_poker::HandIndexer;
use std::iter::FromIterator;

use rust_poker::read_write::VecIO;
use std::fs::{File, OpenOptions};

pub fn cards_to_str(cards: &[u8]) -> String {
    let mut chars: Vec<char> = Vec::new();
    cards.iter().filter(|c| **c < 52).for_each(|c| {
        chars.push(RANK_TO_CHAR[usize::from(*c >> 2)]);
        chars.push(SUIT_TO_CHAR[usize::from(*c & 3)]);
    });
    String::from_iter(chars)
}

fn main() {
    let round = 2;
    let dim = 50;
    let dataset = read_ehs_histograms(round, dim).unwrap();
    let k = 8;
    let indexer = HandIndexer::init(1, [2].to_vec());

    println!("{}", dataset.len_of(Axis(0)));
    println!("{}", dataset.len_of(Axis(1)));

    // // Create new file, exit if file exists
    // // let mut file = OpenOptions::new()
    // //     .write(true)
    // //     .create_new(true)
    // //     .open(format!("emd-abs-r{}-k{}.dat", round, k))
    // //     .unwrap();

    // let mut classifier = HammerlyKmeans::init_pp(k, &dataset, &distance::emd, 25, true);
    // // // println!("intra_cluster_dist: {}", intra_cluster_dist);
    // let inertia = classifier.run(&dataset, &distance::emd, 100, true);
    // println!("inertia: {},", inertia);
    // // // vanilla average with emd 416.81
    // // // hammerly average with emd 413.1409

    // let mut ranges = vec![String::new(); k];
    // let mut cards = [0u8; 2];
    // for i in 0usize..169 {
    //     indexer.get_hand(0, i as u64, &mut cards);
    //     ranges[classifier.assignments[i]] += cards_to_str(&cards).as_str();
    //     ranges[classifier.assignments[i]] += ",";
    // }

    // for i in 0..k {
    //     // println!("");
    //     println!("\"{}\",", ranges[i]);
    // }

    // let assignments: Vec<u32> = classifier.assignments.iter().map(|d| *d as u32).collect();
    // file.write_slice_to_file(&assignments).unwrap();
}
