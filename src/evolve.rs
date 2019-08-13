mod evolution_params;
mod exp_node;

pub use evolution_params::EvolutionParams;
use exp_node::*;
use ordered_float::OrderedFloat;
use rand::prelude::*;
use statrs::distribution::Geometric;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[allow(non_camel_case_types)]
pub type float = f32;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Evolve {
    pop: Vec<Rc<dyn ExpNode>>,
    data: Vec<[float; 2]>,
    params: EvolutionParams,
    total_iterations: usize,
    iters_to_best: usize,
}

#[wasm_bindgen]
impl Evolve {
    pub fn from_xy(xs: Vec<float>, ys: Vec<float>) -> Self {
        Self::new(xs.iter().zip(ys).map(|(&x, y)| [x, y]).collect(), None)
    }

    /// returns how many iterations it took to reach its most fit individual
    pub fn step(&mut self, iterations: usize) {
        let mut rng = rand::thread_rng();

        for _c in 0..iterations {
            let mut new_pop = Vec::with_capacity(self.pop.len());

            // add the best of the last population to new population
            new_pop.push(self.pop[0].clone());

            // add mutations to new population
            while new_pop.len() < self.pop.len() {
                for i in 0..self.pop.len() {
                    if rng.gen::<float>() < (self.pop.len() - i) as float / self.pop.len() as float {
                        for j in 0..self.pop.len() {
                            if j == 0 || rng.gen::<float>() < self.params.repeated_mutation_rate.powf(-(i as float)) {
                                new_pop.push(self.pop[i].mutate(&self.params));
                            } else {
                                break;
                            }
                        }
                    }
                }
                for i in 0..self.pop.len() {
                    if rng.gen::<float>() < (self.params.random_expression_insert_rate as float).powf(-(i as float)) {
                        let depth = Geometric::new(self.params.new_random_expression_prob as _)
                            .unwrap()
                            .sample(&mut rng)
                            - 1.0;
                        new_pop.push(random_expression(depth as _, &self.params));
                    }
                }
            }

            // simplify all of the new population
            for i in 0..new_pop.len() {
                new_pop[i] = new_pop[i].simplify();
            }
            new_pop.sort_by_cached_key(|e| OrderedFloat(e.fitness(&self.data[..])));

            // if we have a better individual, set iterations to best to current iteration
            if new_pop[0].fitness(&self.data[..]) < self.pop[0].fitness(&self.data[..]) {
                self.iters_to_best = self.total_iterations;
            }

            // set new population as current population
            self.pop = new_pop[0..self.pop.len()].to_vec();
            self.total_iterations += 1;

            // if (_c + 1) % 1000 == 0 {
            //     println!(
            //         "iteration {:?}: best size {:?}, best fitness {:?} => {}",
            //         (_c + 1),
            //         self.best_individual().size(),
            //         self.best_fitness(),
            //         self.best_individual()
            //     );
            // }
        }
    }

    pub fn best_fitness(&self) -> float {
        self.pop[0].fitness(&self.data[..])
    }

    pub fn best_eval(&self, x: float) -> float {
        self.pop[0].eval(x)
    }

    pub fn best_string(&self) -> String {
        self.pop[0].to_string()
    }

    pub fn iters_to_best(&self) -> usize {
        self.iters_to_best
    }
}

impl Evolve {
    pub fn new(data: Vec<[float; 2]>, params: Option<EvolutionParams>) -> Self {
        let params = params.unwrap_or_else(|| EvolutionParams::default());
        let mut rng = rand::thread_rng();
        let mut pop = vec![
            {
                let depth = Geometric::new(params.new_random_expression_prob as _)
                    .unwrap()
                    .sample(&mut rng)
                    - 1.0;
                random_expression(depth as _, &params).simplify()
            };
            params.population_num.round() as _
        ];
        pop.sort_by_cached_key(|e| OrderedFloat(e.fitness(&data[..])));

        Self {
            pop,
            data,
            params,
            total_iterations: 0,
            iters_to_best: 0,
        }
    }

    pub fn best_individual(&self) -> Rc<dyn ExpNode> {
        self.pop[0].clone()
    }
}
