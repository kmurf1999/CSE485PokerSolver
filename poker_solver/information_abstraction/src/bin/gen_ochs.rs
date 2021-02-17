use clap::Clap;

use information_abstraction::ochs::gen_ochs_vectors;
use rust_poker::read_write::VecIO;
use std::error::Error;
use std::fs::OpenOptions;
use std::result::Result;

#[derive(Clap)]
#[clap(version = "1.0", author = "Kyle <kmurf1999@gmail.com>")]
struct Opts {
    #[clap(short, long, default_value = "3")]
    round: u8,
    #[clap(short, long, default_value = "1000")]
    sim_count: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    assert!(opts.sim_count > 0);
    assert!(opts.round > 1 && opts.round < 4);

    // create file
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(format!(
            "ochs-vectors-r{}-s{}.dat",
            opts.round, opts.sim_count
        ))?;

    let vectors = gen_ochs_vectors(opts.round, opts.sim_count);

    file.write_slice_to_file(&vectors.as_slice().unwrap())?;
    Ok(())
}
