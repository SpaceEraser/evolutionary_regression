mod expr_parser;

use crate::Float;

pub use expr_parser::{evaluate_expression, ALPHABET};
pub use rand::prelude::*;

#[derive(Clone, Hash, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Expression(Vec<u8>);

impl Expression {
    pub fn new_random(rng: &mut SmallRng, len_range: std::ops::Range<u8>) -> Self {
        let len = rng.gen_range(len_range);
        Expression((0..len).map(|_| ALPHABET.choose(rng).unwrap()).collect())
    }

    pub fn eval(&self, x: Float) -> Float {
        evaluate_expression(&self.0, x)
    }

    pub fn fitness<T>(&self, points: T) -> Float
    where
        T: AsRef<[[Float; 2]]>,
    {
        let squares_sum = points
            .as_ref()
            .into_iter()
            .map(|&[x, y]| (expr_parser::eval(&*self.expr, x) - y).powi(2))
            .sum();

        return squares_sum / (self.expr.len() as Float);
    }

    pub fn mutate(&mut self) {
        todo!()
    }

    pub fn crossover(parents: &[Self]) -> Vec<Self> {
        todo!()
    }
}

impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Safety: `self.expr` is always composed of an allowable alphabet, which is all ASCII
        write!(f, "{}", unsafe {
            std::str::from_utf8_unchecked(&*self.expr)
        })
    }
}


