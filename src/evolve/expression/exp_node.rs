use crate::evolve::{
    evolution_params::EvolutionParams,
    expression::{ExpTree, SIZE_LIMIT},
    float,
};
use approx::relative_eq;
use rand::prelude::*;
use statrs::distribution::{Geometric, Normal};

#[derive(Copy, PartialEq, Clone, PartialOrd, Debug)]
pub enum ExpNodeOp {
    Add,
    Mul,
    Exp,
    Log,
    Sin,
    Var,
    Const(float),
}

impl ExpNodeOp {
    pub fn is_const(self) -> bool {
        use ExpNodeOp::*;
        if let Const(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_nullary(self) -> bool {
        use ExpNodeOp::*;
        self == Var || self.is_const()
    }

    pub fn is_unary(self) -> bool {
        use ExpNodeOp::*;
        self == Sin
    }

    pub fn is_binary(self) -> bool {
        use ExpNodeOp::*;
        [Add, Mul, Exp, Log].iter().any(|&e| e == self)
    }
}

#[derive(Debug, Clone)]
pub struct ExpNode {
    size: u32,
    depth: u32,
    children: Vec<ExpNode>,
    op: ExpNodeOp,
}

impl ExpNode {
    pub fn new_binary(op: ExpNodeOp, a: Self, b: Self) -> Self {
        assert!(op.is_binary());

        Self {
            size: a.size() + b.size() + 1,
            depth: a.depth().max(b.depth()) + 1,
            children: vec![a, b],
            op,
        }
    }

    pub fn new_unary(op: ExpNodeOp, a: Self) -> Self {
        assert!(op.is_unary());

        Self {
            size: a.size() + 1,
            depth: a.depth() + 1,
            children: vec![a],
            op,
        }
    }

    pub fn new_nullary(op: ExpNodeOp) -> Self {
        assert!(op.is_nullary());

        Self {
            size: 1,
            depth: 1,
            children: Vec::new(),
            op,
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn eval(&self, x: float) -> float {
        use ExpNodeOp::*;

        match self.op {
            Add => self.children().iter().map(|n| n.eval(x)).sum(),
            Mul => self
                .children()
                .iter()
                .map(|n| n.eval(x))
                .fold(1.0, |acc, v| acc * v),
            Exp => self.children[0].eval(x).powf(self.children[1].eval(x)),
            Log => self.children[0].eval(x).log(self.children[1].eval(x)),
            Sin => self.children[0].eval(x).sin(),
            Var => x,
            Const(c) => c,
        }
    }

    pub fn children(&self) -> &[ExpNode] {
        &self.children
    }

    /// change node slightly (but call `mutate` on children, which could change them significantly)
    pub fn jitter(&self, tree: &ExpTree, params: &EvolutionParams) -> Self {
        use ExpNodeOp::*;

        let mut rng = rand::thread_rng();

        match self.op {
            Add => ExpNode::new_binary(
                Add,
                self.children[0].mutate(tree, params),
                self.children[1].mutate(tree, params),
            ),
            Mul => ExpNode::new_binary(
                Mul,
                self.children[0].mutate(tree, params),
                self.children[1].mutate(tree, params),
            ),
            Exp => {
                if rand::random::<float>() < params.binary_switch_prob {
                    ExpNode::new_binary(
                        Exp,
                        self.children[1].mutate(tree, params),
                        self.children[0].mutate(tree, params),
                    )
                } else {
                    ExpNode::new_binary(
                        Exp,
                        self.children[0].mutate(tree, params),
                        self.children[1].mutate(tree, params),
                    )
                }
            }
            Log => {
                if rand::random::<float>() < params.binary_switch_prob {
                    ExpNode::new_binary(
                        Log,
                        self.children[1].mutate(tree, params),
                        self.children[0].mutate(tree, params),
                    )
                } else {
                    ExpNode::new_binary(
                        Log,
                        self.children[0].mutate(tree, params),
                        self.children[1].mutate(tree, params),
                    )
                }
            }
            Sin => ExpNode::new_unary(Sin, self.children[0].mutate(tree, params)),
            Var => ExpNode::new_nullary(Var),
            Const(c) => {
                if rng.gen::<float>() < params.const_mutation_prob {
                    let v = c.abs().max(0.0001);
                    let r = Normal::new(0.0, (v / params.const_jitter_factor).into())
                        .unwrap_or_else(|_| {
                            panic!(
                                "invalid: v / const_jitter_factor {}",
                                v / params.const_jitter_factor
                            )
                        })
                        .sample(&mut rng) as float;
                    ExpNode::new_nullary(Const(c + r))
                } else {
                    ExpNode::new_nullary(Const(c))
                }
            }
        }
    }

    /// change node significantly, possibly replacing it entirely
    pub fn mutate(&self, tree: &ExpTree, params: &EvolutionParams) -> Self {
        let mut rng = rand::thread_rng();

        if tree.size() < SIZE_LIMIT
            && rng.gen::<float>() < params.mutate_replace_rate.powf(-(self.size() as float))
        {
            let size = Geometric::new(1.0 / (f64::from(self.size()) + 1.0))
                .unwrap()
                .sample(&mut rng)
                .min(f64::from(SIZE_LIMIT - self.size()));

            random_expression(size as _, params)
        } else {
            self.jitter(tree, params)
        }
    }

    /// return an expression equal but hopefully shorter
    pub fn simplify(&self) -> Self {
        use ExpNodeOp::*;

        let mut simp: Vec<_> = self.children.iter().map(|e| e.simplify()).collect();

        match self.op {
            Add => match (simp[0].op, simp[1].op) {
                (Const(c1), Const(c2)) => ExpNode::new_nullary(Const(c1 + c2)),
                (Const(c1), _) if relative_eq!(c1, 0.0) => simp.remove(1),
                (_, Const(c2)) if relative_eq!(c2, 0.0) => simp.remove(0),
                _ => ExpNode::new_binary(Add, simp.remove(0), simp.remove(0)),
            },
            Mul => match (simp[0].op, simp[1].op) {
                (Const(c1), Const(c2)) => ExpNode::new_nullary(Const(c1 * c2)),
                (Const(c1), _) if relative_eq!(c1, 1.0) => simp.remove(1),
                (_, Const(c2)) if relative_eq!(c2, 1.0) => simp.remove(0),
                _ => ExpNode::new_binary(Mul, simp.remove(0), simp.remove(0)),
            },
            Exp => match (simp[0].op, simp[1].op) {
                (Const(c1), Const(c2)) => ExpNode::new_nullary(Const(c1.powf(c2))),
                (_, Const(c2)) if relative_eq!(c2, 1.0) => simp.remove(0),
                (_, Const(c2)) if relative_eq!(c2, 0.0) => ExpNode::new_nullary(Const(0.0)),
                _ => ExpNode::new_binary(Exp, simp.remove(0), simp.remove(0)),
            },
            Log => match (simp[0].op, simp[1].op) {
                (Const(c1), Const(c2)) => ExpNode::new_nullary(Const(c1.log(c2))),
                _ => ExpNode::new_binary(Log, simp.remove(0), simp.remove(0)),
            },
            Sin => match simp[0].op {
                Const(c1) => ExpNode::new_nullary(Const(c1.sin())),
                _ => ExpNode::new_unary(Sin, simp.remove(0)),
            },
            Var => ExpNode::new_nullary(Var),
            Const(c) => {
                let r = c.round();
                ExpNode::new_nullary(Const(if relative_eq!(c, r) { r } else { c }))
            }
        }
    }
}

impl std::fmt::Display for ExpNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ExpNodeOp::*;
        match self.op {
            Add => write!(f, "({} + {})", self.children[0], self.children[1]),
            Mul => write!(f, "({} * {})", self.children[0], self.children[1]),
            Exp => write!(f, "({} ^ {})", self.children[0], self.children[1]),
            Log => write!(f, "log({}, {})", self.children[0], self.children[1]),
            Sin => write!(f, "sin({})", self.children[0]),
            Var => write!(f, "x"),
            Const(c) => write!(f, "{:.4}", c),
        }
    }
}

pub fn random_expression(mut size: u32, params: &EvolutionParams) -> ExpNode {
    size = size.min(SIZE_LIMIT);

    static BINARY_OPTS: &[fn(u32, &EvolutionParams) -> ExpNode; 4] = &[
        |s, p| {
            let d = thread_rng().gen_range(2, s);
            ExpNode::new_binary(
                ExpNodeOp::Add,
                random_expression(d - 1, p),
                random_expression(s - d, p),
            )
        },
        |s, p| {
            let d = thread_rng().gen_range(2, s);
            ExpNode::new_binary(
                ExpNodeOp::Mul,
                random_expression(d - 1, p),
                random_expression(s - d, p),
            )
        },
        |s, p| {
            let d = thread_rng().gen_range(2, s);
            ExpNode::new_binary(
                ExpNodeOp::Exp,
                random_expression(d - 1, p),
                random_expression(s - d, p),
            )
        },
        |s, p| {
            let d = thread_rng().gen_range(2, s);
            ExpNode::new_binary(
                ExpNodeOp::Log,
                random_expression(d - 1, p),
                random_expression(s - d, p),
            )
        },
    ];
    static UNARY_OPTS: &[fn(u32, &EvolutionParams) -> ExpNode; 1] =
        &[|s, p| ExpNode::new_unary(ExpNodeOp::Sin, random_expression(s - 1, p))];
    static NULLARY_OPTS: &[fn(u32, &EvolutionParams) -> ExpNode; 2] = &[
        |_, _| ExpNode::new_nullary(ExpNodeOp::Var),
        |_, p| {
            ExpNode::new_nullary(ExpNodeOp::Const(
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
    } else {
        panic!("invalid size for new expression: {}", size);
    }

    opts.choose(&mut thread_rng()).unwrap()(size, params)
}
