use crate::evolve::{
    evolution_params::EvolutionParams,
    expression::{random_expression, ExpNode},
    float,
};

#[derive(Debug, Clone)]
pub struct ExpTree {
    root: ExpNode,
}

impl ExpTree {
    pub fn new(root: ExpNode) -> Self {
        Self { root }
    }

    pub fn new_random(size: u32, params: &EvolutionParams) -> Self {
        ExpTree::new(random_expression(size, params))
    }

    pub fn eval(&self, x: float) -> float {
        self.root.eval(x)
    }

    pub fn mutate(&self, params: &EvolutionParams) -> Self {
        Self::new(self.root.mutate(self, params))
    }

    pub fn fitness(&self, data: &[[float; 2]]) -> float {
        self.root.fitness(data)
    }

    pub fn simplify(&self) -> Self {
        ExpTree::new(self.root.simplify())
    }

    pub fn depth(&self) -> u32 {
        self.root.depth()
    }

    pub fn size(&self) -> u32 {
        self.root.size()
    }
}

impl std::fmt::Display for ExpTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.fmt(f)
    }
}
