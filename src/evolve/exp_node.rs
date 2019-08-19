use crate::evolve::evolution_params::EvolutionParams;
use crate::evolve::float;
use approx::relative_eq;
use downcast_rs::{impl_downcast, Downcast};
use rand::prelude::*;
use statrs::distribution::{Geometric, Normal};

const SIZE_LIMIT: i32 = 512;

pub fn random_expression(size: i32, params: &EvolutionParams) -> Box<dyn ExpNode> {
    static BINARY_OPTS: &'static [fn(i32, &EvolutionParams) -> Box<dyn ExpNode>; 4] = &[
        |s, p| {
            let d = thread_rng().gen_range(2, s);
            Box::new(Add::new(
                random_expression(d - 1, p),
                random_expression(s - d, p),
            ))
        },
        |s, p| {
            let d = thread_rng().gen_range(2, s);
            Box::new(Mul::new(
                random_expression(d - 1, p),
                random_expression(s - d, p),
            ))
        },
        |s, p| {
            let d = thread_rng().gen_range(2, s);
            Box::new(Exp::new(
                random_expression(d - 1, p),
                random_expression(s - d, p),
            ))
        },
        |s, p| {
            let d = thread_rng().gen_range(2, s);
            Box::new(Log::new(
                random_expression(d - 1, p),
                random_expression(s - d, p),
            ))
        },
    ];
    static UNARY_OPTS: &'static [fn(i32, &EvolutionParams) -> Box<dyn ExpNode>; 1] =
        &[|s, p| Box::new(Sin::new(random_expression(s - 1, p)))];
    static NULLARY_OPTS: &'static [fn(i32, &EvolutionParams) -> Box<dyn ExpNode>; 2] = &[
        |_, _| Box::new(Variable),
        |_, p| {
            Box::new(Constant(
                Normal::new(p.new_const_mean as _, p.new_const_std as _)
                    .unwrap_or_else(|_| {
                        panic!(
                            "invalid: new_const_mean {} new_const_std {}",
                            p.new_const_mean, p.new_const_std
                        )
                    })
                    .sample(&mut thread_rng()) as float,
            ))
        },
    ];

    let mut opts = Vec::new();

    if size > 1 {
        opts.extend_from_slice(UNARY_OPTS);
        if size > 2 {
            opts.extend_from_slice(BINARY_OPTS);
        }
    } else if size == 1 {
        opts.extend_from_slice(NULLARY_OPTS);
    }

    opts.choose(&mut thread_rng())
        .expect(&format!("invalid size for new expression: {}", size))(size, params)
}

pub trait ExpNode: std::fmt::Debug + std::fmt::Display + Downcast + objekt::Clone {
    fn children(&self) -> Vec<Box<dyn ExpNode>>;

    fn eval(&self, x: float) -> float;

    fn mutate_node(&self, params: &EvolutionParams) -> Box<dyn ExpNode>;

    fn mutate(&self, params: &EvolutionParams) -> Box<dyn ExpNode> {
        let mut rng = rand::thread_rng();

        let self_size = self.size();

        if self_size > SIZE_LIMIT {
            let size = Geometric::new(params.new_random_expression_prob as _)
                .unwrap()
                .sample(&mut rng);

            random_expression(size as _, params)
        } else if rng.gen::<float>() < params.mutate_replace_rate.powf(-(self_size as float)) {
            let size = Geometric::new(1.0 / (self_size as f64 + 0.5))
                .unwrap()
                .sample(&mut rng);

            random_expression(size as _, params)
        } else {
            self.mutate_node(params)
        }
    }

    fn fitness(&self, data: &[[float; 2]]) -> float {
        let size = self.size();
        if size > SIZE_LIMIT {
            return std::f32::MAX;
        }

        let accuracy: float = data
            .iter()
            .map(|&[x, y]| self.eval(x) - y)
            .map(|y| y.abs())
            .sum();

        accuracy + (size as float)
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
        if self.0.is_finite() {
            self.0
        } else {
            0.0
        }
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
        if x.is_finite() {
            x
        } else {
            0.0
        }
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

        if r.is_finite() {
            r
        } else {
            0.0
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

        if r.is_finite() {
            r
        } else {
            0.0
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
