use rand::prelude::*;
use statrs::distribution::Geometric;

fn main() {
    let mut rng = thread_rng();
    for _ in 0..20 {
        let c = Geometric::new(0.1).unwrap().sample(&mut rng);
        dbg!(c);
    }
}
