use crate::evolve::{float, EvolutionParams, Evolve};
use ordered_float::OrderedFloat;
use rand::prelude::*;
use rayon::prelude::*;

static FUNCTIONS: &[fn(float) -> float; 1] = &[
    |x| 2.0 * x.powi(2) - 3.0 * x.powi(3) + 1.0 / (x + 1.0),
    // |x| x.cos() + 1.0,
    // |x| (3.0 as float).powf(x),
    // |x| x * x - x - 1.0,
];
const RUNS_PER_FUNCTION: usize = 10;
const META_POPULATION_NUM: usize = 30;

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct MetaEntity {
    params: EvolutionParams,
    fitness: float,
}

impl MetaEntity {
    /// will cause a slow calculation to take place to calculate fitness
    pub fn from_params(params: EvolutionParams) -> Self {
        let fitness = Self::calculate_fitness(&params);

        Self { params, fitness }
    }

    /// will cause a slow calculation to take place to calculate fitness
    pub fn new_random() -> Self {
        Self::from_params(EvolutionParams::new_random())
    }

    /// will cause a slow calculation to take place to calculate fitness
    pub fn mutate(&self) -> Self {
        Self::from_params(self.params.mutate())
    }

    /// will cause a slow calculation to take place to calculate fitness
    pub fn crossover(entities: &[&Self]) -> Self {
        Self::from_params(EvolutionParams::crossover(
            &entities.iter().map(|me| &me.params).collect::<Vec<_>>(),
        ))
    }

    pub fn fitness(&self) -> float {
        self.fitness
    }

    pub fn params(&self) -> &EvolutionParams {
        &self.params
    }

    fn calculate_fitness(params: &EvolutionParams) -> float {
        let fitness = FUNCTIONS
            .iter()
            .flat_map(|f| (0..RUNS_PER_FUNCTION).map(move |_| f))
            .map(|f| {
                let data: Vec<[float; 2]> = (-5..=5)
                    .map(|i| {
                        let y = f(i as float);
                        [i as float, if y.is_finite() { y } else { 0.0 }]
                    })
                    .collect();
                let mut e = Evolve::new(data, Some(params.clone()));
                e.step(50_000);
                e.best_fitness() * (10_000.0) + (e.iters_to_best() as float)
            })
            .sum::<float>();

        fitness / (FUNCTIONS.len() * RUNS_PER_FUNCTION) as float
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
                .join("\r\n")
                .trim()
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
                .into_par_iter()
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
            let time = chrono::Duration::span(|| {
                let mut new_params = Vec::with_capacity(self.pop.len());

                for i in 0..(self.pop.len() / 2) {
                    if rng.gen::<float>() < (self.pop.len() - i) as float / self.pop.len() as float
                    {
                        new_params.push(self.pop[i].params().mutate());
                    }
                }

                while new_params.len() < self.pop.len() - 1 {
                    let mut parents = Vec::with_capacity(2);
                    for _ in 0..2 {
                        parents.push(self.pop.choose(&mut rng).unwrap().params());
                    }

                    if parents.len() > 1 {
                        new_params.push(EvolutionParams::crossover(&parents));
                    }
                }

                let best = self.best_individual().clone();

                self.pop = new_params
                    .into_par_iter()
                    .map(|p| MetaEntity::from_params(p))
                    .collect();

                self.pop.push(best);

                self.pop.sort_unstable_by_key(|e| OrderedFloat(e.fitness()));
            });

            println!(
                "Built generation {} in {}! Best Indiviual: {}",
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
