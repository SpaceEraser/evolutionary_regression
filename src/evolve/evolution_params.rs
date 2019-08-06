use crate::evolve::float;

#[derive(Copy, PartialEq, Clone, PartialOrd, Debug)]
pub struct EvolutionParams {
    /// [1, inf)
    pub population_num: float,
    /// (-inf, inf)
    pub new_const_mean: float,
    /// (-inf, inf)
    pub new_const_std: float,
    /// (0, 1]
    pub new_random_expression_prob: float,
    /// (1, inf)
    pub repeated_mutation_rate: float,
    /// (1, inf)
    pub random_expression_insert_rate: float,
    /// (1, inf)
    pub mutate_replace_rate: float,
    /// (1, inf)
    pub mutate_random_expression_prob: float,
    /// (0, 1]
    pub const_mutation_prob: float,
    /// [0, 1]
    pub binary_switch_prob: float,
}

impl EvolutionParams {
    pub fn from_array(a: [float; 10]) -> Self {
        Self {
            population_num: a[0],
            new_const_mean: a[1],
            new_const_std: a[2],
            new_random_expression_prob: a[3],
            repeated_mutation_rate: a[4],
            random_expression_insert_rate: a[5],
            mutate_replace_rate: a[6],
            mutate_random_expression_prob: a[7],
            const_mutation_prob: a[8],
            binary_switch_prob: a[9],
        }
    }

    pub fn as_array(&self) -> [float; 10] {
        [
            self.population_num,
            self.new_const_mean,
            self.new_const_std,
            self.new_random_expression_prob,
            self.repeated_mutation_rate,
            self.random_expression_insert_rate,
            self.mutate_replace_rate,
            self.mutate_random_expression_prob,
            self.const_mutation_prob,
            self.binary_switch_prob,
        ]
    }
}

impl Default for EvolutionParams {
    fn default() -> Self {
        EvolutionParams {
            population_num: 50.0,
            new_const_mean: 0.0,
            new_const_std: 2.0,
            new_random_expression_prob: 0.95,
            repeated_mutation_rate: 1.5,
            random_expression_insert_rate: 3.0,
            mutate_replace_rate: 3.0,
            mutate_random_expression_prob: 0.75,
            const_mutation_prob: 0.01,
            binary_switch_prob: 0.01,
        }
    }
}
