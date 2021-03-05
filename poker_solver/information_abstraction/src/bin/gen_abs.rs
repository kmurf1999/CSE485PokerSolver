use clap::Clap;

use information_abstraction::distance;
use information_abstraction::histogram::read_ehs_histograms;
use information_abstraction::ochs::read_ochs_vectors;
use information_abstraction::mpi_kmeans::MPIKmeans;
use mpi::traits::*;
use ndarray::prelude::*;
use std::result::Result;

use std::error::Error;

use rust_poker::read_write::VecIO;
use std::fs::{File, OpenOptions};

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
    max_iter: usize,
}

// static EMD: &'static Fn(&ArrayView1<f32>,&ArrayView1<f32>) -> f32 = &distance::emd;
// static EUCLID: &'static Fn(&ArrayView1<f32>,&ArrayView1<f32>) -> f32 = &distance::euclid;

fn main() -> Result<(), Box<dyn Error>> {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let rank = world.rank();
    let is_root = rank == 0;
    let opts: Opts = Opts::parse();
    assert!(opts.round <= 3);
    assert!(opts.dim > 0);
    assert!(opts.k > 1);
    assert!(opts.n_restarts > 0);
    assert!(opts.max_iter > 0);

    let dataset = match opts.dist_fn.as_str() {
        "emd" => read_ehs_histograms(opts.round, opts.dim)?,
        "ochs" => read_ochs_vectors(opts.round, opts.dim)?,
        _ => panic!("invalid distance fn. Must be either \"emd\" or \"ochs\""),
    }

    let dist_fn = match opts.dist_fn.as_str() {
        "emd" => &distance::emd as &(dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        "ochs" => &distance::euclid as &(dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        _ => panic!("invalid distance fn. Must be either \"emd\" or \"ochse\""),
    };

    // Create new file, exit if file exists
    let mut file: Option<File> = None;
    if is_root {
        file = Some(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(format!(
                    "{}-abs-r{}-k{}-d{}.dat",
                    opts.dist_fn, opts.round, opts.k, opts.dim
                ))?,
        );
    }

    let mut classifier = match opts.init_fn.as_str() {
        "kpp" => MPIKmeans::init_pp(world, opts.k, &dataset, dist_fn, opts.n_restarts, true),
        "random" => MPIKmeans::init_random(world, opts.k, &dataset, dist_fn, opts.n_restarts, true),
        _ => panic!("invalid init fn.  Must be either \"kpp\" or \"random\""),
    };

    classifier.run(&dataset, world, dist_fn, opts.max_iter, true);

    if is_root {
        let assignments: Vec<u32> = classifier.assignments[0..dataset.len_of(Axis(0))]
            .iter()
            .map(|d| *d as u32)
            .collect();
        file.unwrap().write_slice_to_file(&assignments)?;
    }
    Ok(())
}
