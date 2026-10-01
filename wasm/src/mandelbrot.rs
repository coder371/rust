//! Escape-time Mandelbrot renderer.
//!
//! Writes straight into an RGBA buffer that JavaScript reads back out of
//! linear memory — no copying, no per-pixel calls across the boundary.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Mandelbrot {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    /// Iterations actually spent on the last frame, summed over every pixel.
    last_work: u64,
}

#[wasm_bindgen]
impl Mandelbrot {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Mandelbrot {
        Mandelbrot {
            width,
            height,
            pixels: vec![0; width * height * 4],
            last_work: 0,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.pixels.resize(width * height * 4, 0);
    }

    /// `scale` is the width of the viewport in complex-plane units.
    pub fn render(&mut self, center_re: f64, center_im: f64, scale: f64, max_iter: u32) {
        let w = self.width;
        let h = self.height;
        let aspect = h as f64 / w as f64;
        let half_re = scale * 0.5;
        let half_im = scale * aspect * 0.5;
        let step_re = scale / w as f64;
        let step_im = scale * aspect / h as f64;
        let min_re = center_re - half_re;
        let min_im = center_im - half_im;
        let escape = 4.0_f64;
        let mut work: u64 = 0;

        for y in 0..h {
            let ci = min_im + y as f64 * step_im;
            let row = y * w * 4;
            for x in 0..w {
                let cr = min_re + x as f64 * step_re;

                // Skip the two largest interior regions analytically; they are
                // pure black and would otherwise burn the full iteration budget.
                let q = (cr - 0.25) * (cr - 0.25) + ci * ci;
                let in_cardioid = q * (q + (cr - 0.25)) <= 0.25 * ci * ci;
                let in_bulb = (cr + 1.0) * (cr + 1.0) + ci * ci <= 0.0625;

                let (iter, zr2, zi2) = if in_cardioid || in_bulb {
                    (max_iter, 0.0, 0.0)
                } else {
                    let mut zr = 0.0_f64;
                    let mut zi = 0.0_f64;
                    let mut zr2 = 0.0_f64;
                    let mut zi2 = 0.0_f64;
                    let mut i = 0_u32;
                    while i < max_iter && zr2 + zi2 <= escape {
                        zi = 2.0 * zr * zi + ci;
                        zr = zr2 - zi2 + cr;
                        zr2 = zr * zr;
                        zi2 = zi * zi;
                        i += 1;
                    }
                    (i, zr2, zi2)
                };

                work += iter as u64;
                let o = row + x * 4;
                if iter >= max_iter {
                    self.pixels[o] = 6;
                    self.pixels[o + 1] = 7;
                    self.pixels[o + 2] = 20;
                } else {
                    // Continuous escape value removes the banding you get from
                    // colouring by the raw integer count.
                    let mag = zr2 + zi2;
                    let nu = if mag > 1.0 {
                        (0.5 * mag.ln() / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2
                    } else {
                        0.0
                    };
                    let smooth = (iter as f64 + 1.0 - nu).max(0.0);
                    let (r, g, b) = palette(smooth, max_iter);
                    self.pixels[o] = r;
                    self.pixels[o + 1] = g;
                    self.pixels[o + 2] = b;
                }
                self.pixels[o + 3] = 255;
            }
        }
        self.last_work = work;
    }

    pub fn pixels_ptr(&self) -> *const u8 {
        self.pixels.as_ptr()
    }

    pub fn pixels_len(&self) -> usize {
        self.pixels.len()
    }

    /// Total inner-loop iterations spent on the last frame.
    pub fn last_work(&self) -> f64 {
        self.last_work as f64
    }
}

/// A palindromic ramp through the page's own palette, cycled over a fixed
/// span of escape values. Because the ramp starts and ends on the same dark
/// value it wraps without a seam, and because the span is fixed rather than
/// tied to `max_iter` the structure stays legible at any zoom depth.
fn palette(smooth: f64, _max_iter: u32) -> (u8, u8, u8) {
    const SPAN: f64 = 6.2;
    const STOPS: [(f32, f32, f32); 5] = [
        (0.043, 0.051, 0.102), // ink
        (0.165, 0.133, 0.439), // deep indigo
        (0.486, 0.361, 1.000), // violet
        (0.941, 0.706, 0.161), // amber
        (1.000, 0.945, 0.816), // warm white
    ];

    // Square-rooting first spreads the fast-escaping outer region, where most
    // pixels sit, instead of crushing it all into the ramp's darkest few percent.
    let cycle = (smooth.max(0.0).sqrt() / SPAN).fract() as f32;
    // Triangle wave: out to the bright end of the ramp and back again.
    let u = (1.0 - (2.0 * cycle - 1.0).abs()).powf(1.4);

    let scaled = u * (STOPS.len() - 1) as f32;
    let i = (scaled.floor() as usize).min(STOPS.len() - 2);
    let f = scaled - i as f32;
    let (r0, g0, b0) = STOPS[i];
    let (r1, g1, b1) = STOPS[i + 1];

    (
        ((r0 + (r1 - r0) * f) * 255.0) as u8,
        ((g0 + (g1 - g0) * f) * 255.0) as u8,
        ((b0 + (b1 - b0) * f) * 255.0) as u8,
    )
}
