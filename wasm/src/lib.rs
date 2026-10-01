mod basics;
mod bench;
mod life;
mod mandelbrot;
mod nbody;
mod rng;

pub use basics::*;
pub use bench::*;
pub use life::Life;
pub use mandelbrot::Mandelbrot;
pub use nbody::Field;

use wasm_bindgen::prelude::*;

/// Bytes currently committed to the module's linear memory.
///
/// WebAssembly memory grows in 64 KiB pages and never shrinks, so this only
/// ever rises — which is exactly what the memory strip is there to show.
#[wasm_bindgen]
pub fn heap_bytes() -> f64 {
    (core::arch::wasm32::memory_size(0) * 65536) as f64
}
