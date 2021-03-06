mod evolution_params;
mod expression;

use crate::float;

pub use evolution_params::EvolutionParams;
use expression::ExpTree;
use ordered_float::OrderedFloat;
use rand::prelude::*;
use statrs::distribution::Geometric;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Evolve {
    pop: Vec<ExpTree>,
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

    /// step evolution forward
    pub fn step(&mut self, iterations: usize) {
        let mut rng = rand::thread_rng();

        // println!(
        //     "Stepping {} iterations with population of {}",
        //     iterations,
        //     self.pop.len()
        // );

        for _c in 0..iterations {
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
                    if rng.gen::<float>() < (self.pop.len() - i) as float / self.pop.len() as float
                    {
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
                        let size =
                            Geometric::new(f64::from(self.params.new_random_expression_prob))
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

            // if (_c + 1) % 10_000 == 0 {
            //     println!("{}", self);
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
        let params = params.unwrap_or_else(EvolutionParams::default);
        let mut rng = rand::thread_rng();
        let mut pop: Vec<_> = (0..(params.population_num.round() as usize))
            .map(|_| {
                let size = Geometric::new(f64::from(params.new_random_expression_prob))
                    .unwrap()
                    .sample(&mut rng);

                ExpTree::new_random(size as _, &params).simplify()
            })
            .collect();
        pop.sort_by_cached_key(|e| OrderedFloat(e.fitness(&data[..])));

        Self {
            pop,
            data,
            params,
            total_iterations: 0,
            iters_to_best: 0,
        }
    }

    pub fn from_pair(data: Vec<[float; 2]>) -> Self {
        Self::new(data, None)
    }

    pub fn best_individual(&self) -> &ExpTree {
        &self.pop[0]
    }
}

impl std::fmt::Display for Evolve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        writeln!(f, "\ttotal_iterations {}", self.total_iterations)?;
        writeln!(f, "\tpopulation size: {}", self.pop.len())?;
        writeln!(
            f,
            "\tpopulation size (in bytes): {}",
            self.pop
                .iter()
                .map(|e| (e.size() as usize) * std::mem::size_of_val(e))
                .sum::<usize>()
        )?;
        writeln!(
            f,
            "\tmax expression size: {}",
            self.pop.iter().map(|e| e.size()).max().unwrap()
        )?;
        writeln!(
            f,
            "\tmax expression depth: {}",
            self.pop.iter().map(|e| e.depth()).max().unwrap()
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
