use crate::evolve::float;
use rand::distributions::OpenClosed01;
use rand::prelude::*;
use statrs::distribution::{Exponential, Geometric, Normal};

#[derive(PartialEq, Clone, PartialOrd, Debug)]
pub struct EvolutionParams {
    /// valid range: [1, inf)
    pub population_num: float,

    /// valid range: (-inf, inf)
    pub new_const_mean: float,

    /// valid range: (0, inf)
    pub new_const_std: float,

    /// valid range: (0, 1]
    pub new_random_expression_prob: float,

    /// valid range: (1, inf)
    pub repeated_mutation_rate: float,

    /// valid range: (1, inf)
    pub random_expression_insert_rate: float,

    /// valid range: (1, inf)
    pub mutate_replace_rate: float,

    /// valid range: (0, 1]
    pub const_mutation_prob: float,

    /// valid range: [1, inf)
    pub const_jitter_factor: float,

    /// valid range: [0, 1]
    pub binary_switch_prob: float,
}

impl EvolutionParams {
    pub fn is_valid(&self) -> bool {
        use std::ops::{Bound::*, RangeBounds};

        (1.0..).contains(&self.population_num)
            && (0.0..).contains(&self.new_const_std)
            && (Excluded(0.0), Included(1.0)).contains(&self.new_random_expression_prob)
            && self.repeated_mutation_rate > 1.0
            && self.random_expression_insert_rate > 1.0
            && self.mutate_replace_rate > 1.0
            && (Excluded(0.0), Included(1.0)).contains(&self.const_mutation_prob)
            && (1.0..).contains(&self.const_jitter_factor)
            && (0.0..=1.0).contains(&self.binary_switch_prob)
    }

    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            population_num: Geometric::new(0.1 as _).unwrap().sample(&mut rng) as _,
            new_const_mean: Normal::new(0.0, 1.0).unwrap().sample(&mut rng) as _,
            new_const_std: Exponential::new(0.9 as _).unwrap().sample(&mut rng) as _,
            new_random_expression_prob: rng.sample(OpenClosed01),
            repeated_mutation_rate: (Exponential::new(0.5 as _).unwrap().sample(&mut rng) as float)
                + 1.0,
            random_expression_insert_rate: (Exponential::new(0.5 as _).unwrap().sample(&mut rng)
                as float)
                + 1.0,
            mutate_replace_rate: (Exponential::new(0.5 as _).unwrap().sample(&mut rng) as float)
                + 1.0,
            const_mutation_prob: rng.sample(OpenClosed01),
            const_jitter_factor: (Exponential::new(0.5 as _).unwrap().sample(&mut rng) as float)
                + 1.0,
            binary_switch_prob: rng.sample(OpenClosed01),
        }
    }

    pub fn mutate(&self) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            population_num: {
                let o = Normal::new(0.0, self.population_num as _)
                    .unwrap()
                    .sample(&mut rng) as float;
                (self.population_num + o).max(1.0)
            },
            new_const_mean: {
                let o = Normal::new(0.0, 1.0).unwrap().sample(&mut rng) as float;
                self.new_const_mean + o
            },
            new_const_std: {
                let o = Normal::new(0.0, 1.0).unwrap().sample(&mut rng) as float;
                (self.new_const_std + o).max(0.0001)
            },
            new_random_expression_prob: {
                let o = Normal::new(0.0, 1.0).unwrap().sample(&mut rng) as float;
                (self.new_random_expression_prob + o).clamp(0.0001, 1.0)
            },
            repeated_mutation_rate: {
                let o = Normal::new(0.0, 1.0).unwrap().sample(&mut rng) as float;
                (self.repeated_mutation_rate + o).max(1.0001)
            },
            random_expression_insert_rate: {
                let o = Normal::new(0.0, 1.0).unwrap().sample(&mut rng) as float;
                (self.random_expression_insert_rate + o).max(1.0001)
            },
            mutate_replace_rate: {
                let o = Normal::new(0.0, 1.0).unwrap().sample(&mut rng) as float;
                (self.mutate_replace_rate + o).max(1.0001)
            },
            const_mutation_prob: {
                let o = Normal::new(0.0, 1.0).unwrap().sample(&mut rng) as float;
                (self.const_mutation_prob + o).clamp(0.0001, 1.0)
            },
            const_jitter_factor: {
                let o = Normal::new(0.0, 1.0).unwrap().sample(&mut rng) as float;
                (self.const_jitter_factor + o).max(1.0)
            },
            binary_switch_prob: {
                let o = Normal::new(0.0, 1.0).unwrap().sample(&mut rng) as float;
                (self.binary_switch_prob + o).clamp(0.0, 1.0)
            },
        }
    }

    pub fn crossover(entities: &[&Self]) -> Self {
        let mut rng = rand::thread_rng();
        let param_arr: Vec<_> = (0..EvolutionParams::num_params())
            .map(|i| entities.choose(&mut rng).unwrap().as_array()[i])
            .collect();
        Self::from_array(&param_arr)
    }

    pub fn from_array(a: &[float]) -> Self {
        Self {
            population_num: a[0],
            new_const_mean: a[1],
            new_const_std: a[2],
            new_random_expression_prob: a[3],
            repeated_mutation_rate: a[4],
            random_expression_insert_rate: a[5],
            mutate_replace_rate: a[6],
            const_mutation_prob: a[7],
            const_jitter_factor: a[8],
            binary_switch_prob: a[9],
        }
    }

    pub fn as_array(&self) -> Box<[float; 10]> {
        Box::new([
            self.population_num,
            self.new_const_mean,
            self.new_const_std,
            self.new_random_expression_prob,
            self.repeated_mutation_rate,
            self.random_expression_insert_rate,
            self.mutate_replace_rate,
            self.const_mutation_prob,
            self.const_jitter_factor,
            self.binary_switch_prob,
        ])
    }

    pub fn num_params() -> usize {
        10
    }
}

impl Default for EvolutionParams {
    fn default() -> Self {
        EvolutionParams {
            population_num: 50.0,
            new_const_mean: 0.0,
            new_const_std: 2.0,
            new_random_expression_prob: 0.3,
            repeated_mutation_rate: 1.5,
            random_expression_insert_rate: 3.0,
            mutate_replace_rate: 3.0,
            const_mutation_prob: 0.01,
            const_jitter_factor: 3.0,
            binary_switch_prob: 0.01,
        }
    }
}
