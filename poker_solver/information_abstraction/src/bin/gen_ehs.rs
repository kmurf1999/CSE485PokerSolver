/// Generates an Expected Hand Strength (EHS) table.
/// Table is used to aid the creation of state abstractions for each betting round
///
/// Using indicies obtained from rust_poker::HandIndexer object
/// Lookup the EHS of any hand
use information_abstraction::ehs::generate_ehs_table;
use clap::Clap;

#[derive(Clap)]
#[clap(version = "1.0", author = "Kyle <kmurf1999@gmail.com>")]
struct Opts {
    #[clap(short, long, default_value = "4")]
    n_threads: usize,
}

fn main() {
    let opts: Opts = Opts::parse();
    generate_ehs_table(opts.n_threads);
}
