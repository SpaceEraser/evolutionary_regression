#![feature(box_patterns)]
#![feature(clamp)]

#[allow(non_camel_case_types)]
pub type float = f32;

pub mod evolve;
pub mod meta_evolve;

use wasm_bindgen::prelude::*;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen]
pub fn from_xy(xs: Vec<float>, ys: Vec<float>) -> evolve::Evolve {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&JsValue::from_str("Setup panic hook."));

    return evolve::Evolve::from_xy(xs, ys);
}
