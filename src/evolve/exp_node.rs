use crate::evolve::evolution_params::EvolutionParams;
use crate::evolve::float;
use approx::relative_eq;
use downcast_rs::{impl_downcast, Downcast};
use rand::prelude::*;
use statrs::distribution::{Geometric, Normal};

pub fn random_expression(mut size: i32, params: &EvolutionParams) -> Box<dyn ExpNode> {
    size = size.min(100);
    let mut rng = rand::thread_rng();

    if size == 1 {
        if rng.gen::<bool>() {
            Box::new(Variable)
        } else {
            Box::new(Constant(
                Normal::new(params.new_const_mean as _, params.new_const_std as _)
                    .unwrap_or_else(|_| {
                        panic!(
                            "invalid: new_const_mean {} new_const_std {}",
                            params.new_const_mean, params.new_const_std
                        )
                    })
                    .sample(&mut rng) as float,
            ))
        }
    } else if size > 2 {
        match rng.gen_range::<i32, _, _>(0, 4) {
            0 => Box::new(Add::new(
                random_expression(size - 2, params),
                random_expression(size - 2, params),
            )),
            1 => Box::new(Mul::new(
                random_expression(size - 2, params),
                random_expression(size - 2, params),
            )),
            2 => Box::new(Exp::new(
                random_expression(size - 2, params),
                random_expression(size - 2, params),
            )),
            3 => Box::new(Log::new(
                random_expression(size - 2, params),
                random_expression(size - 2, params),
            )),
            _ => panic!("this should never happen"),
        }
    } else if size > 1 {
        Box::new(Sin::new(random_expression(size - 1, params)))
    } else {
        panic!("invalid size for random_expression: {:?}", size);
    }
}

pub trait ExpNode: std::fmt::Debug + std::fmt::Display + Downcast + objekt::Clone {
    fn children(&self) -> Vec<Box<dyn ExpNode>>;

    fn eval(&self, x: float) -> float;

    fn mutate_node(&self, params: &EvolutionParams) -> Box<dyn ExpNode>;

    fn mutate(&self, params: &EvolutionParams) -> Box<dyn ExpNode> {
        let mut rng = rand::thread_rng();

        let size = self.size() as float;

        if size < 100.0 && rng.gen::<float>() < params.mutate_replace_rate.powf(-size) {
            let size = Geometric::new(1.0 / (size as f64 + 0.01).sqrt())
                .unwrap()
                .sample(&mut rng).min((size*1.1).ceil() as _);
            random_expression(size as _, params)
        } else {
            self.mutate_node(params)
        }
    }

    fn fitness(&self, data: &[[float; 2]]) -> float {
        let accuracy: float = data
            .iter()
            .map(|&[x, y]| self.eval(x) - y)
            .map(|y| y.abs())
            .sum();

        accuracy + (self.size() as float)
    }

    fn simplify(&self) -> Box<dyn ExpNode>;

    fn depth(&self) -> i32 {
        self.children().iter().map(|c| c.depth()).max().unwrap_or(0) + 1
    }

    fn size(&self) -> i32 {
        self.children().iter().map(|c| c.size()).sum::<i32>() + 1
    }
}
impl_downcast!(ExpNode);
objekt::clone_trait_object!(ExpNode);

//---------------------------------------------------------------

#[derive(Copy, PartialEq, Clone, PartialOrd, Debug)]
pub struct Constant(float);

impl ExpNode for Constant {
    fn children(&self) -> Vec<Box<dyn ExpNode>> {
        vec![]
    }

    fn eval(&self, _: float) -> float {
        self.0
    }

    fn mutate_node(&self, params: &EvolutionParams) -> Box<dyn ExpNode> {
        let mut rng = rand::thread_rng();

        if rng.gen::<float>() < params.const_mutation_prob {
            let c = self.0.abs().max(0.0001);
            let r = Normal::new(0.0, (c / params.const_jitter_factor).into())
                .unwrap_or_else(|_| {
                    panic!(
                        "invalid: c/const_jitter_factor {}",
                        c / params.const_jitter_factor
                    )
                })
                .sample(&mut rng) as float;
            Box::new(Constant(self.0 + r))
        } else {
            Box::new(Constant(self.0))
        }
    }

    fn simplify(&self) -> Box<dyn ExpNode> {
        Box::new(Constant(self.0))
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

//---------------------------------------------------------------

#[derive(Copy, Eq, PartialEq, Clone, Ord, PartialOrd, Debug, Hash)]
pub struct Variable;

impl ExpNode for Variable {
    fn children(&self) -> Vec<Box<dyn ExpNode>> {
        vec![]
    }

    fn eval(&self, x: float) -> float {
        x
    }

    fn mutate_node(&self, _: &EvolutionParams) -> Box<dyn ExpNode> {
        Box::new(Variable)
    }

    fn simplify(&self) -> Box<dyn ExpNode> {
        Box::new(Variable)
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x")
    }
}

//---------------------------------------------------------------

// #[derive(Eq, PartialEq, Clone, Ord, PartialOrd, Debug, Hash)]
#[derive(Clone, Debug)]
pub struct Add(Box<dyn ExpNode>, Box<dyn ExpNode>);

impl Add {
    pub fn new(a: Box<dyn ExpNode>, b: Box<dyn ExpNode>) -> Self {
        Self(a, b)
    }
}

impl ExpNode for Add {
    fn children(&self) -> Vec<Box<dyn ExpNode>> {
        vec![self.0.clone(), self.1.clone()]
    }

    fn eval(&self, x: float) -> float {
        self.0.eval(x) + self.1.eval(x)
    }

    fn mutate_node(&self, params: &EvolutionParams) -> Box<dyn ExpNode> {
        Box::new(Add(self.0.mutate(params), self.1.mutate(params)))
    }

    fn simplify(&self) -> Box<dyn ExpNode> {
        let a = self.0.simplify();
        let b = self.1.simplify();

        match (a.downcast_ref::<Constant>(), b.downcast_ref::<Constant>()) {
            (Some(Constant(c1)), Some(Constant(c2))) => Box::new(Constant(c1 + c2)),
            (Some(Constant(c)), None) if relative_eq!(*c, 0.0) => b,
            (None, Some(Constant(c))) if relative_eq!(*c, 0.0) => a,
            _ => Box::new(Add(a, b)),
        }
    }
}

impl std::fmt::Display for Add {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} + {})", self.0, self.1)
    }
}

//---------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Mul(Box<dyn ExpNode>, Box<dyn ExpNode>);

impl Mul {
    pub fn new(a: Box<dyn ExpNode>, b: Box<dyn ExpNode>) -> Self {
        Self(a, b)
    }
}

impl ExpNode for Mul {
    fn children(&self) -> Vec<Box<dyn ExpNode>> {
        vec![self.0.clone(), self.1.clone()]
    }

    fn eval(&self, x: float) -> float {
        self.0.eval(x) * self.1.eval(x)
    }

    fn mutate_node(&self, params: &EvolutionParams) -> Box<dyn ExpNode> {
        Box::new(Mul(self.0.mutate(params), self.1.mutate(params)))
    }

    fn simplify(&self) -> Box<dyn ExpNode> {
        let a = self.0.simplify();
        let b = self.1.simplify();

        match (a.downcast_ref::<Constant>(), b.downcast_ref::<Constant>()) {
            (Some(Constant(c1)), Some(Constant(c2))) => Box::new(Constant(c1 * c2)),
            (Some(Constant(c)), None) if relative_eq!(*c, 1.0) => b,
            (None, Some(Constant(c))) if relative_eq!(*c, 1.0) => a,
            _ => Box::new(Mul(a, b)),
        }
    }
}

impl std::fmt::Display for Mul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} * {})", self.0, self.1)
    }
}

//---------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Sin(Box<dyn ExpNode>);

impl Sin {
    pub fn new(a: Box<dyn ExpNode>) -> Self {
        Self(a)
    }
}

impl ExpNode for Sin {
    fn children(&self) -> Vec<Box<dyn ExpNode>> {
        vec![self.0.clone()]
    }

    fn eval(&self, x: float) -> float {
        self.0.eval(x).sin()
    }

    fn mutate_node(&self, params: &EvolutionParams) -> Box<dyn ExpNode> {
        Box::new(Sin(self.0.mutate(params)))
    }

    fn simplify(&self) -> Box<dyn ExpNode> {
        let a = self.0.simplify();

        if let Some(Constant(c)) = a.downcast_ref::<Constant>() {
            Box::new(Constant(c.sin()))
        } else {
            Box::new(Sin(a))
        }
    }
}

impl std::fmt::Display for Sin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sin({})", self.0)
    }
}

//---------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Exp(Box<dyn ExpNode>, Box<dyn ExpNode>);

impl Exp {
    pub fn new(a: Box<dyn ExpNode>, b: Box<dyn ExpNode>) -> Self {
        Self(a, b)
    }
}

impl ExpNode for Exp {
    fn children(&self) -> Vec<Box<dyn ExpNode>> {
        vec![self.0.clone(), self.1.clone()]
    }

    fn eval(&self, x: float) -> float {
        let a = self.0.eval(x);
        let b = self.1.eval(x);
        let r = a.powf(b);

        if !r.is_finite() {
            0.0
        } else {
            r
        }
    }

    fn mutate_node(&self, params: &EvolutionParams) -> Box<dyn ExpNode> {
        if rand::random::<float>() < params.binary_switch_prob {
            Box::new(Exp(self.1.mutate(params), self.0.mutate(params)))
        } else {
            Box::new(Exp(self.0.mutate(params), self.1.mutate(params)))
        }
    }

    fn simplify(&self) -> Box<dyn ExpNode> {
        let a = self.0.simplify();
        let b = self.1.simplify();

        match (a.downcast_ref::<Constant>(), b.downcast_ref::<Constant>()) {
            (Some(Constant(c1)), Some(Constant(c2))) => {
                let r = c1.powf(*c2);

                Box::new(Constant(if !r.is_finite() { 0.0 } else { r }))
            }
            (None, Some(Constant(c))) if relative_eq!(*c, 1.0) => a,
            (None, Some(Constant(c))) if relative_eq!(*c, 0.0) => Box::new(Constant(1.0)),
            _ => Box::new(Exp(a, b)),
        }
    }
}

impl std::fmt::Display for Exp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} ^ {})", self.0, self.1)
    }
}

//---------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Log(Box<dyn ExpNode>, Box<dyn ExpNode>);

impl Log {
    pub fn new(a: Box<dyn ExpNode>, b: Box<dyn ExpNode>) -> Self {
        Self(a, b)
    }
}

impl ExpNode for Log {
    fn children(&self) -> Vec<Box<dyn ExpNode>> {
        vec![self.0.clone(), self.1.clone()]
    }

    fn eval(&self, x: float) -> float {
        let a = self.0.eval(x);
        let b = self.1.eval(x);
        let r = a.log(b);

        if !r.is_finite() {
            0.0
        } else {
            r
        }
    }

    fn mutate_node(&self, params: &EvolutionParams) -> Box<dyn ExpNode> {
        if rand::random::<float>() < params.binary_switch_prob {
            Box::new(Log(self.1.mutate(params), self.0.mutate(params)))
        } else {
            Box::new(Log(self.0.mutate(params), self.1.mutate(params)))
        }
    }

    fn simplify(&self) -> Box<dyn ExpNode> {
        let a = self.0.simplify();
        let b = self.1.simplify();

        match (a.downcast_ref::<Constant>(), b.downcast_ref::<Constant>()) {
            (Some(Constant(c1)), Some(Constant(c2))) => {
                let r = c1.log(*c2);

                Box::new(Constant(if !r.is_finite() { 0.0 } else { r }))
            }
            _ => Box::new(Log(a, b)),
        }
    }
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} ^ {})", self.0, self.1)
    }
}
