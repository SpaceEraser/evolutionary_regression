use crate::evolve::{float, EvolutionParams, Evolve};
use ordered_float::OrderedFloat;
use rand::prelude::*;
use rayon::prelude::*;
use statrs::distribution::{Exponential, Normal};
use std::cell::RefCell;

static FUNCTIONS: &'static [fn(float) -> float; 3] = &[
    |x| 2.0 * x * x - 3.0 * x * x * x,
    |x| x.sin() + 1.0,
    |x| (3.0 as float).powf(x),
];

#[derive(PartialEq, Clone, PartialOrd, Debug)]
pub struct MetaEntity {
    params: EvolutionParams,
    fitness: RefCell<Option<float>>,
}

impl MetaEntity {
    pub fn new_random() -> Self {
        Self {
            params: EvolutionParams::new_random(),
            fitness: RefCell::new(None),
        }
    }

    pub fn mutate(&self) -> Self {
        Self {
            params: self.params.mutate(),
            fitness: RefCell::new(None),
        }
    }

    pub fn crossover(entities: &[&Self]) -> Self {
        MetaEntity {
            params: EvolutionParams::crossover(&entities.iter().map(|me| &me.params).collect::<Vec<_>>()),
            fitness: RefCell::new(None),
        }
    }

    pub fn fitness(&self) -> float {
        if !self.params.is_valid() {
            return std::f64::INFINITY as float;
        }
        if let Some(fitness) = *self.fitness.borrow() {
            return fitness;
        }

        // calculate fitness in parallel
        let params = &self.params;
        let fitness = FUNCTIONS
            .iter()
            .flat_map(|f| (0..3).map(move |_| f))
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|f| {
                let data: Vec<[float; 2]> = (-5..=5).map(|i| [i as float, f(i as float)]).collect();
                let mut e = Evolve::new(data.clone(), Some(params.clone()));
                e.step(50_000);
                e.best_fitness() * (10_000.0) + (e.iters_to_best() as float)
            })
            .sum::<float>();
        let fitness = fitness / (FUNCTIONS.len() * 3) as float;

        *self.fitness.borrow_mut() = Some(fitness);
        fitness
    }
}

#[derive(Debug)]
pub struct MetaEvolve {
    pop: Vec<MetaEntity>,
    total_iterations: usize,
}

impl MetaEvolve {
    pub fn new() -> Self {
        Self {
            pop: (0..30).map(|_| MetaEntity::new_random()).collect(),
            total_iterations: 0,
        }
    }

    pub fn step(&mut self, iterations: usize) {
        let mut rng = rand::thread_rng();

        for _c in 0..iterations {
            let mut new_pop = Vec::with_capacity(self.pop.len());

            new_pop.push(self.pop[0].clone());

            for i in 0..self.pop.len() / 2 {
                if rng.gen::<float>() < (self.pop.len() - i) as float / self.pop.len() as float {
                    new_pop.push(self.pop[i].mutate());
                }
            }

            while new_pop.len() < self.pop.len() {
                let mut parents = Vec::new();
                for _ in 0..2 {
                    parents.push(self.pop.choose(&mut rng).unwrap());
                }

                if parents.len() > 1 {
                    new_pop.push(MetaEntity::crossover(&parents[..]));
                }
            }

            println!("Sorting generation {}", _c + 1);

            new_pop.sort_unstable_by_key(|e| OrderedFloat(e.fitness()));
            self.pop = new_pop;

            println!("Sorted generation {}! Best fitness: {}", _c + 1, self.best_fitness());

            self.total_iterations += 1;
        }
    }

    pub fn best_fitness(&self) -> float {
        self.pop[0].fitness()
    }

    pub fn best_individual(&self) -> &EvolutionParams {
        &self.pop[0].params
    }
}
