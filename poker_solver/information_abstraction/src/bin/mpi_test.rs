fn main() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();
    let root_process = world.process_at_rank(0);

    let indexes = (0..169).iter().collect();
    let mut batch = Vec::new();
    if rank == 0 {
        root_process.scatter_into_root(&indexes, &mut batch);
    } else {
        root_process.scatter_into(&mut batch);
    }

    println!("rank: {}, batch: {:?}", rank, batch);
}
