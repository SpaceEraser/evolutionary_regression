#![feature(box_patterns)]
#![feature(bind_by_move_pattern_guards)]

use rand::prelude::*;
use ordered_float::OrderedFloat;
use statrs::distribution::{Geometric, Normal};
use approx::relative_eq;

#[derive(PartialEq, Clone, PartialOrd, Debug)]
pub enum ExpNode {
    Variable,
    Constant(f64),
    Add(Box<ExpNode>, Box<ExpNode>),
    Mul(Box<ExpNode>, Box<ExpNode>),
}

impl ExpNode {
    pub fn random(depth: i32) -> Self {
        use ExpNode::*;

        let mut rng = rand::thread_rng();

        if depth == 0 {
            match rng.gen::<bool>() {
                true => Variable,
                false => Constant(Normal::new(0.0, 2.0).unwrap().sample(&mut rng)),
            }
        } else {
            match rng.gen::<bool>() {
                true => Add(Box::new(Self::random(depth-1)), Box::new(Self::random(depth-1))),
                false => Mul(Box::new(Self::random(depth-1)), Box::new(Self::random(depth-1))),
            }
        }
    }

    pub fn depth(&self) -> i32 {
        use ExpNode::*;

        match self {
            Variable => 0,
            Constant(_) => 0,
            Add(a, b) | Mul(a, b) => a.depth().max(b.depth())+1,
        }
    }

    pub fn simplify(&self) -> Self {
        use ExpNode::*;

        match self {
            Variable => Variable,
            &Constant(c) => Constant(c),
            &Add(box Constant(a), box Constant(b)) => Constant(a+b),
            &Mul(box Constant(a), box Constant(b)) => Constant(a*b),
            &Add(ref a, box Constant(x)) | &Add(box Constant(x), ref a) if relative_eq!(x, 0.0) => (**a).clone(),
            &Mul(ref a, box Constant(x)) | &Mul(box Constant(x), ref a) if relative_eq!(x, 1.0) => (**a).clone(),
            &Add(ref a, ref b) => {
                let _a = a.simplify();
                let _b = b.simplify();
                if _a != **a || _b != **b { Add(Box::new(_a), Box::new(_b)).simplify() } else { Add(Box::new(_a), Box::new(_b)) }
            },
            &Mul(ref a, ref b) => {
                let _a = a.simplify();
                let _b = b.simplify();
                if _a != **a || _b != **b { Mul(Box::new(_a), Box::new(_b)).simplify() } else { Mul(Box::new(_a), Box::new(_b)) }
            },
        }
    }

    pub fn eval(&self, x: f64) -> f64 {
        use ExpNode::*;

        match self {
            Variable => x,
            &Constant(c) => c,
            Add(a, b) => a.eval(x) + b.eval(x),
            Mul(a, b) => a.eval(x) * b.eval(x),
        }
    }

    // lower is better
    pub fn fitness(&self, data: &[(f64, f64)]) -> f64 {
        data.iter().map(|&(x, y)| self.eval(x) - y).map(|y| y.abs()).sum()
    }

    pub fn mutate(&self) -> Self {
        use ExpNode::*;

        let mut rng = rand::thread_rng();

        if rand::random::<f64>() < (3_f64).powf(-(self.depth()+1) as f64) {
            Self::random(Geometric::new(0.75).unwrap().sample(&mut rng) as _)
        } else {
            match self {
                Variable => self.clone(),
                &Constant(c) => {
                    if rand::random::<f64>() < 0.005 {
                        let c = c.max(0.001);
                        let r = Normal::new(0.0, c.abs()/3.0).unwrap().sample(&mut rng);
                        Constant(c+r)
                    } else {
                        Constant(c)
                    }
                },
                Add(a, b) => Add(Box::new(a.mutate()), Box::new(b.mutate())),
                Mul(a, b) => Mul(Box::new(a.mutate()), Box::new(b.mutate())),
            }
        }
    }
}

impl std::fmt::Display for ExpNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ExpNode::*;
        match self {
            Variable => write!(f, "x"),
            Constant(c) => write!(f, "{:2.2}", c),
            Add(a, b) => write!(f, "({} + {})", a, b),
            Mul(a, b) => write!(f, "({} * {})", a, b),
        }
    }
}

pub fn evolve(data: &[(f64, f64)]) -> ExpNode {
    const POPULATION_NUM: usize = 40;
    let mut rng = rand::thread_rng();
    let mut pop = vec![ExpNode::random(Geometric::new(0.95).unwrap().sample(&mut rng) as _); POPULATION_NUM];
    for _ in 0..10_000 {
        pop.sort_by_cached_key(|e| OrderedFloat(e.fitness(data)));

        // for i in 0..POPULATION_NUM {
        //     pop[i] = pop[i].simplify();
        // }

        if relative_eq!(pop[0].fitness(data), 0.0) { break }
        
        let mut new_pop = Vec::new();
        
        // copy the past generation directly
        for i in 0..POPULATION_NUM/10 {
            if rand::random::<f64>() < (2_f64).powf(-(i as f64)) {
                new_pop.push(pop[i].clone());
            }
        }

        // add mutations
        while new_pop.len() < pop.len() {
            for i in 0..POPULATION_NUM {
                if rand::random::<f64>() < (2_f64).powf(-(i as f64)) {
                    new_pop.push(pop[i].mutate());
                }
            }
        }

        pop = new_pop[0..pop.len()].to_vec();
    }

    return pop[0].clone();
}

fn main() {
    let data: Vec<(f64, f64)> = (-25 ..= 25).map(|i| (i as f64, (2*i*i + 3*i*i*i) as f64)).collect();
    println!("the function is approx {}", evolve(&data[..]));
}
