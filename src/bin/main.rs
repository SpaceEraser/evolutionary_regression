fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("{}", evolutionary_regression::expr_parser::eval("1 1.1+", 2.0));

    // evolutionary_regression::find_sol(&[[-1.0, -1.0], [0.0, 0.0], [1.0, 1.0]]);
    evolutionary_regression::find_sol(
        (-10..10)
            .map(|i| i as f32)
            .map(|x| [x, (x * x).cos() - x.sin() + 1.0])
            // .map(|x| [x, x.cos()])
            .collect::<Vec<_>>()
            .as_ref(),
    );

    Ok(())
}
