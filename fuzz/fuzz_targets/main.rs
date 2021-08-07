#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let alphabet = evolutionary_regression::expr_parser::ALPHABET;

    for &d in data {
        if alphabet.into_iter().find(|&&a| a == d).is_none() {
            return;
        }
    }

    assert!(!evolutionary_regression::expr_parser::eval(data, 0.0).is_nan());
});
