use oxigen::*;
use rand::prelude::*;

pub mod expr_parser;

#[derive(Clone)]
struct Expression<'a> {
    expr: Vec<u8>,
    points: &'a [[f32; 2]],
}

impl<'a> Expression<'a> {
    pub fn fitness_raw(&self) -> f32 {
        let mut sum = 0.0;
        let squares = self
            .points
            .into_iter()
            .map(|&[x, y]| (expr_parser::eval(&*self.expr, x) - y).powi(2));

        for s in squares {
            if s.is_nan() {
                return f32::INFINITY;
            }
            sum += s as f32;
        }

        return sum / (self.expr.len() as f32);
    }
}

impl<'a> std::fmt::Display for Expression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&*self.expr))
    }
}

impl<'a> Genotype<u8> for Expression<'a> {
    type ProblemSize = (std::ops::Range<u8>, &'a [[f32; 2]]);
    // type GenotypeHash = u64;

    fn iter(&self) -> std::slice::Iter<u8> {
        self.expr.iter()
    }

    fn into_iter(self) -> std::vec::IntoIter<u8> {
        self.expr.into_iter()
    }

    fn from_iter<I: Iterator<Item = u8>>(&mut self, iter: I) {
        self.expr = iter.collect()
    }

    fn generate(size: &Self::ProblemSize) -> Self {
        let mut rng = SmallRng::from_entropy();
        let len = rng.gen_range(size.0.start, size.0.end);
        Expression {
            expr: (0..len)
                .map(|_| *expr_parser::ALPHABET.choose(&mut rng).unwrap())
                .collect(),
            points: size.1,
        }
    }

    fn fitness(&self) -> f64 {
        (self.fitness_raw() as f64) * 100.0 + (self.expr.len() as f64)
    }

    fn mutate(&mut self, rng: &mut SmallRng, index: usize) {
        self.expr[index] = *expr_parser::ALPHABET.choose(rng).unwrap();
    }

    fn is_solution(&self, _fitness: f64) -> bool {
        // _fitness.is_finite()
        _fitness - (self.expr.len() as f64) < 0.001
    }

    fn distance(&self, other: &Self) -> f64 {
        triple_accel::levenshtein(&*self.expr, &*other.expr) as f64
    }

    // fn hash(&self) -> Self::GenotypeHash {
    //     let mut hasher = std::collections::hash_map::DefaultHasher::new();
    //     self.expr.hash(&mut hasher);
    //     hasher.finish()
    // }
}

pub fn find_sol(points: &[[f32; 2]]) {
    // let progress_log = File::create("progress.csv").expect("Error creating progress log file");
    // let population_log =
    //     File::create("population.txt").expect("Error creating population log file");

    let (mut solutions, generation, progress, _population) =
        GeneticExecution::<u8, Expression>::new()
            .population_size(100)
            .genotype_size((1..30, points))
            // .mutation_rate(Box::new(MutationRates::Linear(SlopeParams {
            //     start: 15.0,
            //     bound: 0.005,
            //     coefficient: -0.005,
            // })))
            // .selection_rate(Box::new(SelectionRates::Linear(SlopeParams {
            //     start: 4.0,
            //     bound: 2.0,
            //     coefficient: -0.0005,
            // })))
            // .select_function(Box::new(SelectionFunctions::Cup))
            // .crossover_function(Box::new(CrossoverFunctions::UniformCross))
            // .age_function(Box::new(AgeFunctions::Quadratic(
            //     AgeThreshold(2),
            //     AgeSlope(4_f64),
            // )))
            // .population_refitness_function(Box::new(PopulationRefitnessFunctions::Niches(
            //     NichesAlpha(1.0),
            //     Box::new(NichesBetaRates::Constant(1.0)),
            //     NichesSigma(0.2),
            // )))
            // .survival_pressure_function(Box::new(
            //     SurvivalPressureFunctions::CompetitiveOverpopulation(M::new(48, 32, 64)),
            // ))
            // .progress_log(10, progress_log)
            // .population_log(50, population_log)
            .stop_criterion(Box::new(StopCriteria::Generation(1_000)))
            // .stop_criterion(Box::new(StopCriteria::SolutionFound))
            .run();

    println!(
        "Finished. Generation: {}. Progress: {}",
        generation, progress
    );
    solutions.sort_unstable_by(|a, b| a.fitness().partial_cmp(&b.fitness()).unwrap());
    for sol in &solutions {
        println!("{}: {}", sol.fitness(), sol);
    }
}
