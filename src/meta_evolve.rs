use crate::evolve::{float, EvolutionParams, Evolve};
use ordered_float::OrderedFloat;
use rand::prelude::*;
use rayon::prelude::*;
use parking_lot::RwLock;

static FUNCTIONS: &[fn(float) -> float; 4] = &[
    |x| 2.0 * x * x - 3.0 * x * x * x,
    |x| x.cos() + 1.0,
    |x| (3.0 as float).powf(x),
    |x| x * x - x - 1.0,
];
const RUNS_PER_FUNCTION: usize = 3;
const META_POPULATION_NUM: usize = 30;

#[derive(Debug)]
pub struct MetaEntity {
    params: EvolutionParams,
    fitness: RwLock<Option<float>>,
}

impl MetaEntity {
    pub fn new_random() -> Self {
        Self {
            params: EvolutionParams::new_random(),
            fitness: RwLock::new(None),
        }
    }

    pub fn mutate(&self) -> Self {
        Self {
            params: self.params.mutate(),
            fitness: RwLock::new(None),
        }
    }

    pub fn crossover(entities: &[&Self]) -> Self {
        MetaEntity {
            params: EvolutionParams::crossover(
                &entities.iter().map(|me| &me.params).collect::<Vec<_>>(),
            ),
            fitness: RwLock::new(None),
        }
    }

    pub fn calculate_fitness(&self) {
        let params = &self.params;
        let fitness = FUNCTIONS
            .iter()
            .flat_map(|f| (0..RUNS_PER_FUNCTION).map(move |_| f))
            .map(|f| {
                let data: Vec<[float; 2]> = (-5..=5).map(|i| [i as float, f(i as float)]).collect();
                let mut e = Evolve::new(data, Some(params.clone()));
                e.step(50_000);
                e.best_fitness() * (10_000.0) + (e.iters_to_best() as float)
            })
            .sum::<float>();
        let fitness = fitness / (FUNCTIONS.len() * RUNS_PER_FUNCTION) as float;

        *self.fitness.write() = Some(fitness);
    }

    pub fn fitness(&self) -> float {
        if let Some(fitness) = *self.fitness.read() {
            return fitness;
        }

        self.calculate_fitness();
        
        (*self.fitness.read()).unwrap()
    }
}

impl Clone for MetaEntity {
    fn clone(&self) -> Self {
        Self {
            params: self.params.clone(),
            fitness: RwLock::new(None),
        }
    }
}

impl std::fmt::Display for MetaEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        writeln!(f, "\tfitness: {:.4}", self.fitness())?;
        writeln!(
            f,
            "\tparams: {}",
            self.params
                .to_string()
                .lines()
                .map(|l| format!("\t{}", l))
                .collect::<Vec<_>>()
                .join("\r\n").trim()
        )?;
        write!(f, "}}")
    }
}

#[derive(Debug)]
pub struct MetaEvolve {
    pop: Vec<MetaEntity>,
    total_iterations: usize,
}

impl Default for MetaEvolve {
    fn default() -> Self {
        Self {
            pop: (0..META_POPULATION_NUM)
                .map(|_| MetaEntity::new_random())
                .collect(),
            total_iterations: 0,
        }
    }
}

impl MetaEvolve {
    pub fn step(&mut self, iterations: usize) {
        let mut rng = rand::thread_rng();

        for c in 0..iterations {
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

            let time = chrono::Duration::span(|| {
                new_pop.par_iter().for_each(|e| e.calculate_fitness());
                new_pop.sort_unstable_by_key(|e| OrderedFloat(e.fitness()));
                self.pop = new_pop;
            });

            println!(
                "Sorted generation {} in {}s! Best Indiviual: {}",
                c + 1,
                time,
                self.best_individual()
            );

            self.total_iterations += 1;
        }
    }

    pub fn best_fitness(&self) -> float {
        self.pop[0].fitness()
    }

    pub fn best_params(&self) -> &EvolutionParams {
        &self.pop[0].params
    }

    pub fn best_individual(&self) -> &MetaEntity {
        &self.pop[0]
    }
}
