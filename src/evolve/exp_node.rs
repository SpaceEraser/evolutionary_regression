use crate::evolve::evolution_params::EvolutionParams;
use crate::evolve::float;
use approx::relative_eq;
use downcast_rs::{impl_downcast, Downcast};
use rand::prelude::*;
use statrs::distribution::{Geometric, Normal};
use std::rc::Rc;

pub fn random_expression(size: i32, params: &EvolutionParams) -> Rc<dyn ExpNode> {
    let size = size.min(100);

    let mut rng = rand::thread_rng();

    if size == 1 {
        if rng.gen::<bool>() {
            Rc::new(Variable)
        } else {
            Rc::new(Constant(
                Normal::new(params.new_const_mean as _, params.new_const_std as _)
                    .expect(&format!(
                        "invalid: new_const_mean {} new_const_std {}",
                        params.new_const_mean, params.new_const_std
                    ))
                    .sample(&mut rng) as float,
            ))
        }
    } else if size > 2 {
        match rng.gen_range::<i32, _, _>(0, 4) {
            0 => Rc::new(Add::new(
                random_expression(size - 2, params),
                random_expression(size - 2, params),
            )),
            1 => Rc::new(Mul::new(
                random_expression(size - 2, params),
                random_expression(size - 2, params),
            )),
            2 => Rc::new(Exp::new(
                random_expression(size - 2, params),
                random_expression(size - 2, params),
            )),
            3 => Rc::new(Log::new(
                random_expression(size - 2, params),
                random_expression(size - 2, params),
            )),
            _ => panic!("this should never happen"),
        }
    } else if size > 1 {
        Rc::new(Sin::new(random_expression(size - 1, params)))
    } else {
        panic!("invalid size for random_expression: {:?}", size);
    }
}

pub trait ExpNode: std::fmt::Debug + std::fmt::Display + Downcast {
    fn children(&self) -> Vec<Rc<dyn ExpNode>>;

    fn eval(&self, x: float) -> float;

    fn mutate_node(&self, params: &EvolutionParams) -> Rc<dyn ExpNode>;

    fn mutate(&self, params: &EvolutionParams) -> Rc<dyn ExpNode> {
        let mut rng = rand::thread_rng();

        let size = self.size();

        if rng.gen::<float>() < params.mutate_replace_rate.powf(-size as float) {
            let size = Geometric::new(1.0 / ((size + 1) as f64))
                .unwrap()
                .sample(&mut rng);
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

    fn simplify(&self) -> Rc<dyn ExpNode>;

    fn depth(&self) -> i32 {
        self.children().iter().map(|c| c.depth()).max().unwrap_or(0) + 1
    }

    fn size(&self) -> i32 {
        self.children().iter().map(|c| c.size()).sum::<i32>() + 1
    }
}
impl_downcast!(ExpNode);

//---------------------------------------------------------------

#[derive(Copy, PartialEq, Clone, PartialOrd, Debug)]
pub struct Constant(float);

impl ExpNode for Constant {
    fn children(&self) -> Vec<Rc<dyn ExpNode>> {
        vec![]
    }

    fn eval(&self, _: float) -> float {
        self.0
    }

    fn mutate_node(&self, params: &EvolutionParams) -> Rc<dyn ExpNode> {
        let mut rng = rand::thread_rng();

        if rng.gen::<float>() < params.const_mutation_prob {
            let c = self.0.abs().max(0.0001);
            let r = Normal::new(0.0, (c / params.const_jitter_factor).into())
                .expect(&format!(
                    "invalid: c/const_jitter_factor {}",
                    c / params.const_jitter_factor
                ))
                .sample(&mut rng) as float;
            Rc::new(Constant(self.0 + r))
        } else {
            Rc::new(Constant(self.0))
        }
    }

    fn simplify(&self) -> Rc<dyn ExpNode> {
        Rc::new(Constant(self.0))
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
    fn children(&self) -> Vec<Rc<dyn ExpNode>> {
        vec![]
    }

    fn eval(&self, x: float) -> float {
        x
    }

    fn mutate_node(&self, _: &EvolutionParams) -> Rc<dyn ExpNode> {
        Rc::new(Variable)
    }

    fn simplify(&self) -> Rc<dyn ExpNode> {
        Rc::new(Variable)
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
pub struct Add(Rc<dyn ExpNode>, Rc<dyn ExpNode>);

impl Add {
    pub fn new(a: Rc<dyn ExpNode>, b: Rc<dyn ExpNode>) -> Self {
        Self(a, b)
    }
}

impl ExpNode for Add {
    fn children(&self) -> Vec<Rc<dyn ExpNode>> {
        vec![self.0.clone(), self.1.clone()]
    }

    fn eval(&self, x: float) -> float {
        self.0.eval(x) + self.1.eval(x)
    }

    fn mutate_node(&self, params: &EvolutionParams) -> Rc<dyn ExpNode> {
        Rc::new(Add(self.0.mutate(params), self.1.mutate(params)))
    }

    fn simplify(&self) -> Rc<dyn ExpNode> {
        let a = self.0.simplify();
        let b = self.1.simplify();

        match (a.downcast_ref::<Constant>(), b.downcast_ref::<Constant>()) {
            (Some(Constant(c1)), Some(Constant(c2))) => Rc::new(Constant(c1 + c2)),
            (Some(Constant(c)), None) if relative_eq!(*c, 0.0) => b,
            (None, Some(Constant(c))) if relative_eq!(*c, 0.0) => a,
            _ => Rc::new(Add(a, b)),
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
pub struct Mul(Rc<dyn ExpNode>, Rc<dyn ExpNode>);

impl Mul {
    pub fn new(a: Rc<dyn ExpNode>, b: Rc<dyn ExpNode>) -> Self {
        Self(a, b)
    }
}

impl ExpNode for Mul {
    fn children(&self) -> Vec<Rc<dyn ExpNode>> {
        vec![self.0.clone(), self.1.clone()]
    }

    fn eval(&self, x: float) -> float {
        self.0.eval(x) * self.1.eval(x)
    }

    fn mutate_node(&self, params: &EvolutionParams) -> Rc<dyn ExpNode> {
        Rc::new(Mul(self.0.mutate(params), self.1.mutate(params)))
    }

    fn simplify(&self) -> Rc<dyn ExpNode> {
        let a = self.0.simplify();
        let b = self.1.simplify();

        match (a.downcast_ref::<Constant>(), b.downcast_ref::<Constant>()) {
            (Some(Constant(c1)), Some(Constant(c2))) => Rc::new(Constant(c1 * c2)),
            (Some(Constant(c)), None) if relative_eq!(*c, 1.0) => b,
            (None, Some(Constant(c))) if relative_eq!(*c, 1.0) => a,
            _ => Rc::new(Mul(a, b)),
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
pub struct Sin(Rc<dyn ExpNode>);

impl Sin {
    pub fn new(a: Rc<dyn ExpNode>) -> Self {
        Self(a)
    }
}

impl ExpNode for Sin {
    fn children(&self) -> Vec<Rc<dyn ExpNode>> {
        vec![self.0.clone()]
    }

    fn eval(&self, x: float) -> float {
        self.0.eval(x).sin()
    }

    fn mutate_node(&self, params: &EvolutionParams) -> Rc<dyn ExpNode> {
        Rc::new(Sin(self.0.mutate(params)))
    }

    fn simplify(&self) -> Rc<dyn ExpNode> {
        let a = self.0.simplify();

        if let Some(Constant(c)) = a.downcast_ref::<Constant>() {
            Rc::new(Constant(c.sin()))
        } else {
            Rc::new(Sin(a))
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
pub struct Exp(Rc<dyn ExpNode>, Rc<dyn ExpNode>);

impl Exp {
    pub fn new(a: Rc<dyn ExpNode>, b: Rc<dyn ExpNode>) -> Self {
        Self(a, b)
    }
}

impl ExpNode for Exp {
    fn children(&self) -> Vec<Rc<dyn ExpNode>> {
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

    fn mutate_node(&self, params: &EvolutionParams) -> Rc<dyn ExpNode> {
        if rand::random::<float>() < params.binary_switch_prob {
            Rc::new(Exp(self.1.mutate(params), self.0.mutate(params)))
        } else {
            Rc::new(Exp(self.0.mutate(params), self.1.mutate(params)))
        }
    }

    fn simplify(&self) -> Rc<dyn ExpNode> {
        let a = self.0.simplify();
        let b = self.1.simplify();

        match (a.downcast_ref::<Constant>(), b.downcast_ref::<Constant>()) {
            (Some(Constant(c1)), Some(Constant(c2))) => {
                let r = c1.powf(*c2);

                Rc::new(Constant(if !r.is_finite() { 0.0 } else { r }))
            }
            (None, Some(Constant(c))) if relative_eq!(*c, 1.0) => a,
            (None, Some(Constant(c))) if relative_eq!(*c, 0.0) => Rc::new(Constant(1.0)),
            _ => Rc::new(Exp(a, b)),
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
pub struct Log(Rc<dyn ExpNode>, Rc<dyn ExpNode>);

impl Log {
    pub fn new(a: Rc<dyn ExpNode>, b: Rc<dyn ExpNode>) -> Self {
        Self(a, b)
    }
}

impl ExpNode for Log {
    fn children(&self) -> Vec<Rc<dyn ExpNode>> {
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

    fn mutate_node(&self, params: &EvolutionParams) -> Rc<dyn ExpNode> {
        if rand::random::<float>() < params.binary_switch_prob {
            Rc::new(Log(self.1.mutate(params), self.0.mutate(params)))
        } else {
            Rc::new(Log(self.0.mutate(params), self.1.mutate(params)))
        }
    }

    fn simplify(&self) -> Rc<dyn ExpNode> {
        let a = self.0.simplify();
        let b = self.1.simplify();

        match (a.downcast_ref::<Constant>(), b.downcast_ref::<Constant>()) {
            (Some(Constant(c1)), Some(Constant(c2))) => {
                let r = c1.log(*c2);

                Rc::new(Constant(if !r.is_finite() { 0.0 } else { r }))
            }
            _ => Rc::new(Log(a, b)),
        }
    }
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} ^ {})", self.0, self.1)
    }
}
