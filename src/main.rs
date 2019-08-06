#![feature(box_patterns)]
#![feature(bind_by_move_pattern_guards)]

mod evolve;
use evolve::float;

fn main() {
    let data: Vec<[float; 2]> = (-5..=5)
        .map(|i| [i as float, (2*i*i + 3*i*i*i) as float])
        // .map(|i| [i as float, (i as float).sin()+1.0])
        // .map(|i| [i as float, (3.0 as float).powf(i as float)])
        .collect();
    let mut e = evolve::Evolve::from_pair(data);
    e.step(50_000);
    println!("the function is approx {}", e.best_individual());
}
