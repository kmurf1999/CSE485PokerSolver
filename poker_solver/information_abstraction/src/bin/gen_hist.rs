use clap::Clap;
use information_abstraction::histogram::generate_ehs_histograms;
/// Generates Expected Hand Strength (EHS) histograms
///
/// These histograms are used as features for clustering hands using K-Means.
/// These clusters make up an information abstraction which is used by our counterfactual regret minimization algorithm
///
use std::error::Error;
use std::result::Result;

#[derive(Clap)]
#[clap(version = "1.0", author = "Kyle <kmurf1999@gmail.com>")]
struct Opts {
    round: usize,
    dim: usize,
}

/// Generates a dataset to run a hand clustering algorithm on
fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    assert!(opts.round < 4);

    generate_ehs_histograms(opts.round, opts.dim)?;

    Ok(())
}
