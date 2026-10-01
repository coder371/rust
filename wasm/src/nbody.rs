//! Particle field with a software rasteriser.
//!
//! Every particle is integrated and splatted to the framebuffer inside
//! WebAssembly. JavaScript issues exactly one `putImageData` per frame
//! regardless of whether there are 2,000 particles or 60,000.

use wasm_bindgen::prelude::*;

use crate::rng::Rng;

const MAX_ATTRACTORS: usize = 4;

/// Mass the caller is expected to place at the centre of the field. `scatter`
/// seeds velocities for it so the disc starts out in balance rather than
/// collapsing on the first frame.
pub const CORE_MASS: f32 = 2_400_000.0;

#[wasm_bindgen]
pub struct Field {
    width: usize,
    height: usize,
    px: Vec<f32>,
    py: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,
    pixels: Vec<u8>,
    attractors: Vec<f32>, // x, y, mass triples
    trail: u8,
    mean_speed: f32,
    rng: Rng,
}

#[wasm_bindgen]
impl Field {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize, count: usize) -> Field {
        let mut field = Field {
            width,
            height,
            px: vec![0.0; count],
            py: vec![0.0; count],
            vx: vec![0.0; count],
            vy: vec![0.0; count],
            pixels: vec![0; width * height * 4],
            attractors: Vec::with_capacity(MAX_ATTRACTORS * 3),
            trail: 24,
            mean_speed: 0.0,
            rng: Rng::new(0xC0FFEE),
        };
        field.scatter();
        field
    }

    /// Rebuilds the population as a rotating disc, which settles into
    /// spiral arms instead of collapsing straight to the centre.
    pub fn scatter(&mut self) {
        let cx = self.width as f32 * 0.5;
        let cy = self.height as f32 * 0.5;
        let radius = (self.width.min(self.height) as f32) * 0.44;

        for i in 0..self.px.len() {
            let a = self.rng.unit() * std::f32::consts::TAU;
            let r = radius * self.rng.unit().sqrt();
            let x = cx + a.cos() * r;
            let y = cy + a.sin() * r;
            self.px[i] = x;
            self.py[i] = y;
            // Circular-orbit speed for the core the caller keeps at the centre.
            // Inner particles lap the outer ones, and that shear is what draws
            // the arms out of an initially featureless disc.
            let orbit = (CORE_MASS / r.max(24.0)).sqrt();
            self.vx[i] = -a.sin() * orbit + self.rng.signed() * 0.25;
            self.vy[i] = a.cos() * orbit + self.rng.signed() * 0.25;
        }
        self.pixels.iter_mut().for_each(|p| *p = 0);
    }

    pub fn set_count(&mut self, count: usize) {
        self.px.resize(count, 0.0);
        self.py.resize(count, 0.0);
        self.vx.resize(count, 0.0);
        self.vy.resize(count, 0.0);
        self.scatter();
    }

    pub fn set_trail(&mut self, trail: u8) {
        self.trail = trail;
    }

    pub fn clear_attractors(&mut self) {
        self.attractors.clear();
    }

    pub fn add_attractor(&mut self, x: f32, y: f32, mass: f32) {
        if self.attractors.len() < MAX_ATTRACTORS * 3 {
            self.attractors.push(x);
            self.attractors.push(y);
            self.attractors.push(mass);
        }
    }

    pub fn step(&mut self, dt: f32) {
        let n = self.px.len();
        let w = self.width as f32;
        let h = self.height as f32;
        let mut speed_sum = 0.0_f32;

        for i in 0..n {
            let mut ax = 0.0_f32;
            let mut ay = 0.0_f32;
            let x = self.px[i];
            let y = self.py[i];

            let mut a = 0;
            while a + 2 < self.attractors.len() {
                let dx = self.attractors[a] - x;
                let dy = self.attractors[a + 1] - y;
                let mass = self.attractors[a + 2];
                // Softened so a particle passing through a well does not
                // acquire infinite velocity.
                let d2 = dx * dx + dy * dy + 240.0;
                let inv = 1.0 / (d2 * d2.sqrt());
                ax += dx * mass * inv;
                ay += dy * mass * inv;
                a += 3;
            }

            let mut vx = (self.vx[i] + ax * dt) * 0.9995;
            let mut vy = (self.vy[i] + ay * dt) * 0.9995;
            let mut nx = x + vx * dt;
            let mut ny = y + vy * dt;

            // Reflect at the walls so the population stays on screen.
            if nx < 0.0 {
                nx = -nx;
                vx = -vx * 0.86;
            } else if nx >= w {
                nx = 2.0 * w - nx - 1.0;
                vx = -vx * 0.86;
            }
            if ny < 0.0 {
                ny = -ny;
                vy = -vy * 0.86;
            } else if ny >= h {
                ny = 2.0 * h - ny - 1.0;
                vy = -vy * 0.86;
            }

            self.px[i] = nx;
            self.py[i] = ny;
            self.vx[i] = vx;
            self.vy[i] = vy;
            speed_sum += vx * vx + vy * vy;
        }

        self.mean_speed = if n > 0 { (speed_sum / n as f32).sqrt() } else { 0.0 };
    }

    pub fn render(&mut self) {
        // Fade the previous frame; the residue is what draws the trails.
        let fade = self.trail;
        for p in self.pixels.chunks_exact_mut(4) {
            p[0] = p[0].saturating_sub(fade);
            p[1] = p[1].saturating_sub(fade);
            p[2] = p[2].saturating_sub(fade.saturating_add(2));
        }

        let w = self.width;
        for i in 0..self.px.len() {
            let x = self.px[i] as usize;
            let y = self.py[i] as usize;
            if x >= w || y >= self.height {
                continue;
            }
            // Fast particles read amber, slow ones violet.
            let sp = (self.vx[i] * self.vx[i] + self.vy[i] * self.vy[i]).sqrt();
            let t = (sp / 220.0).min(1.0);
            let r = (70.0 + 170.0 * t) as u8;
            let g = (52.0 + 128.0 * t) as u8;
            let b = (240.0 - 190.0 * t) as u8;

            let o = (y * w + x) * 4;
            self.pixels[o] = self.pixels[o].saturating_add(r);
            self.pixels[o + 1] = self.pixels[o + 1].saturating_add(g);
            self.pixels[o + 2] = self.pixels[o + 2].saturating_add(b);
            self.pixels[o + 3] = 255;
        }

        // Alpha must stay opaque everywhere or the faded trail turns milky.
        for p in self.pixels.chunks_exact_mut(4) {
            p[3] = 255;
        }
    }

    pub fn pixels_ptr(&self) -> *const u8 {
        self.pixels.as_ptr()
    }
    pub fn pixels_len(&self) -> usize {
        self.pixels.len()
    }
    pub fn count(&self) -> usize {
        self.px.len()
    }
    /// Bytes held by the four position/velocity arrays.
    pub fn state_bytes(&self) -> usize {
        self.px.len() * 4 * 4
    }
    pub fn mean_speed(&self) -> f32 {
        self.mean_speed
    }
    /// The mass the caller should hold at the centre of the field.
    pub fn core_mass(&self) -> f32 {
        CORE_MASS
    }
}
