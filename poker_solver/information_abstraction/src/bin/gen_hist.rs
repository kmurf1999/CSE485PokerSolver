/// Generates Expected Hand Strength (EHS) histograms
/// 
/// These histograms are used as features for clustering hands using K-Means.
/// These clusters make up an information abstraction which is used by our counterfactual regret minimization algorithm
/// 
use information_abstraction::histogram::generate_ehs_histograms;
use clap::Clap;
use std::result::Result;
use std::error::Error;

#[derive(Clap)]
#[clap(version = "1.0", author = "Kyle <kmurf1999@gmail.com>")]
struct Opts {
    #[clap(short, long, default_value = "4")]
    n_threads: usize,
    round: usize,
    dim: usize,
    n_samples: usize,
}

/// Generates a dataset to run a hand clustering algorithm on
fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    assert!(opts.round < 5);
    assert!(opts.n_threads > 0);
    generate_ehs_histograms(opts.n_threads, opts.round, opts.dim, opts.n_samples)?;
    Ok(())
}
