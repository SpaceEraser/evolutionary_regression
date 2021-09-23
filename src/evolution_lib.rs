use crate::expression_lib::{evaluate_expression, Expression};
use crate::Float;

use ordered_float::OrderedFloat;
use rand::prelude::*;

#[derive(Copy, Clone)]
struct EvolutionParams {
    mutation_rate: Float,
    genocide_delay: usize,
    population_num: usize,
}

impl Default for EvolutionParams {
    fn default() -> Self {
        EvolutionParams {
            mutation_rate: 0.1,
            genocide_delay: 50,
            population_num: 100
        }
    }
}

#[derive(Debug, Clone)]
pub struct Evolve {
    current_population: Vec<ExpTree>,
    input_points: Vec<[Float; 2]>,
    evolution_parameters: EvolutionParams,
    rng: SmallRng,
    total_iterations: usize,
}

impl Evolve {
    pub fn new(input_points: Vec<[float; 2]>, params: Option<EvolutionParams>) -> Self {
        let evolution_params = params.unwrap_or_default();
        let mut rng = SmallRng::from_entropy();
        let mut initial_population: Vec<_> = (0..evolution_params.population_num)
            .map(|_| Expression::new_random(&mut rng, 1..30))
            .collect();
        initial_population.sort_by_cached_key(|e| OrderedFloat(e.fitness(&input_points)));

        Evolve {
            current_population: initial_population,
            input_points,
            evolution_parameters,
            rng,
            total_iterations: 0,
        }
    }

    /// step evolution forward
    pub fn step(&mut self) {
        let pop_size = self
            .pop
            .iter()
            .map(|e| (e.size() as usize) * std::mem::size_of_val(e))
            .sum::<usize>();
        if pop_size > 1_000_000 {
            println!("Huge population size detected: {}", self);
        }
        let mut new_pop = Vec::with_capacity(self.pop.len());

        // add the best of the last population to new population
        new_pop.push(self.pop[0].clone());

        // add mutations to new population
        'newloop: while new_pop.len() < self.pop.len() {
            for i in 0..self.pop.len() {
                if rng.gen::<float>() < (self.pop.len() - i) as float / self.pop.len() as float {
                    for j in 0..self.pop.len() {
                        if j == 0
                            || rng.gen::<float>()
                                < self.params.repeated_mutation_rate.powf(-(i as float))
                        {
                            new_pop.push(self.pop[i].mutate(&self.params));

                            if new_pop.len() == self.pop.len() {
                                break 'newloop;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
            for i in 0..self.pop.len() {
                if rng.gen::<float>()
                    < (self.params.random_expression_insert_rate as float).powf(-(i as float))
                {
                    let size = Geometric::new(f64::from(self.params.new_random_expression_prob))
                        .unwrap()
                        .sample(&mut rng);

                    new_pop.push(ExpTree::new_random(size as _, &self.params));
                    if new_pop.len() == self.pop.len() {
                        break 'newloop;
                    }
                }
            }
        }

        // simplify all of the new population
        for tree in &mut new_pop {
            *tree = tree.simplify();
        }
        new_pop.sort_by_cached_key(|e| OrderedFloat(e.fitness(&self.data[..])));

        // if we have a better individual, set iterations to best to current iteration
        if new_pop[0].fitness(&self.data[..]) < self.pop[0].fitness(&self.data[..]) {
            self.iters_to_best = self.total_iterations;
        }

        // set new population as current population
        self.pop = new_pop;
        self.total_iterations += 1;
    }
}

impl std::fmt::Display for Evolve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        writeln!(f, "\ttotal_iterations {}", self.total_iterations)?;
        writeln!(f, "\tpopulation size: {}", self.pop.len())?;
        writeln!(
            f,
            "\tmax expression size: {}",
            self.pop.iter().map(|e| e.size()).max().unwrap()
        )?;
        writeln!(
            f,
            "\tbest expression size: {}",
            self.best_individual().size()
        )?;
        writeln!(
            f,
            "\tbest expression depth: {}",
            self.best_individual().depth()
        )?;
        writeln!(f, "\tbest expression fitness: {}", self.best_fitness())?;
        writeln!(f, "\tbest expression:  {}", self.best_individual())?;
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
