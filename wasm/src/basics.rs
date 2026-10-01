//! The first three functions this project started with, kept as the smallest
//! possible example of the boundary.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("مرحبا {}! 👋", name)
}

/// `u64` because 13! already overflows 32 bits.
#[wasm_bindgen]
pub fn factorial(n: u32) -> f64 {
    (1..=n.min(20) as u64).product::<u64>() as f64
}
