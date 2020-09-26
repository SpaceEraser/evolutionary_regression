use evolutionary_regression::float;
use evolutionary_regression::evolve::*;
// use rayon::prelude::*;

fn main() {
    let data: Vec<[float; 2]> = (-5..=5)
        .map(|i| [i as float, (2 * i * i - 3 * i * i * i) as float])
        // .map(|i| [i as float, (i as float).cos() + 1.0])
        // .map(|i| [i as float, (3.0 as float).powi(i)])
        .collect();

    // (0..1000).into_par_iter().for_each(|_| {
    let mut e = Evolve::new(
        data.clone(),
        Some(EvolutionParams::from_array(&[
            8.2905, -1.3461, 1.9842, 1.0, 6.0611, 2.6694, 1.0001, 0.0001, 5.6295, 0.0,
        ])),
    );
    e.step(50_000);
    println!("the function is approx {}", e.best_individual());
    // });
}
