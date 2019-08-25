use evolutionary_regression::meta_evolve::MetaEvolve;

fn main() {
    let mut m = MetaEvolve::default();
    m.step(100);
    dbg!(m.best_individual());
}
