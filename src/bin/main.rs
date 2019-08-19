use evolutionary_regression::meta_evolve::MetaEvolve;
use rayon;

fn main() {
    // increase stack size
    rayon::ThreadPoolBuilder::new()
        .stack_size(4 * 1024 * 1024)
        .build_global()
        .unwrap();

    let mut m = MetaEvolve::new();
    m.step(100);
    dbg!(m.best_individual());
}
