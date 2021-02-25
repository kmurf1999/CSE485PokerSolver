use clap::Clap;

use information_abstraction::distance;
use information_abstraction::histogram::read_ehs_histograms;
use information_abstraction::kmeans::{HammerlyKmeans, Kmeans};
use ndarray::prelude::*;
use std::result::Result;
use std::error::Error;

use rust_poker::read_write::VecIO;
use std::fs::OpenOptions;


#[derive(Clap)]
#[clap(version = "1.0", author = "Kyle <kmurf1999@gmail.com>")]
struct Opts {
    #[clap(long)]
    round: usize,
    #[clap(long)]
    k: usize,
    #[clap(long)]
    dim: usize,
    #[clap(long, default_value = "emd")]
    dist_fn: String,
    #[clap(long, default_value = "kpp")]
    init_fn: String,
    #[clap(long, default_value = "1")]
    n_restarts: usize,
    #[clap(long, default_value = "100")]
    max_iter: usize

}

// static EMD: &'static Fn(&ArrayView1<f32>,&ArrayView1<f32>) -> f32 = &distance::emd;
// static EUCLID: &'static Fn(&ArrayView1<f32>,&ArrayView1<f32>) -> f32 = &distance::euclid;

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    assert!(opts.round <= 3);
    assert!(opts.dim > 0);
    assert!(opts.k > 1);
    assert!(opts.n_restarts > 0);
    assert!(opts.max_iter > 0);
    let dataset = read_ehs_histograms(opts.round, opts.dim)?;

    let dist_fn = match opts.dist_fn.as_str() {
        "emd" => &distance::emd as &(dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        "euclid" => &distance::euclid as &(dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        _ => panic!("invalid distance fn. Must be either \"emd\" or \"euclid\"")
    };

    // Create new file, exit if file exists
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(format!("emd-abs-r{}-k{}-d{}.dat", opts.round, opts.k, opts.dim))?;

    
    let mut classifier = match opts.init_fn.as_str() {
        "kpp" => HammerlyKmeans::init_pp(opts.k, &dataset, dist_fn, opts.n_restarts, true),
        "random" => HammerlyKmeans::init_random(opts.k, &dataset, dist_fn, opts.n_restarts, true),
        _ => panic!("invalid init fn.  Must be either \"kpp\" or \"random\"")
    };

    classifier.run(&dataset, dist_fn, opts.max_iter, true);

    let assignments: Vec<u32> = classifier.assignments.iter().map(|d| *d as u32).collect();
    file.write_slice_to_file(&assignments)?;
    Ok(())
}