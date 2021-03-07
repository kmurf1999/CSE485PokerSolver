/// Generates an Expected Hand Strength (EHS) table.
/// Table is used to aid the creation of state abstractions for each betting round
///
/// Using indicies obtained from rust_poker::HandIndexer object
/// Lookup the EHS of any hand
use information_abstraction::ehs::generate_ehs_table;

fn main() {
    generate_ehs_table();
}
