//! Conway's Game of Life on a wrapping torus.
//!
//! The neighbour count is computed with an offset table rather than modular
//! arithmetic per neighbour, which is what makes a 320x320 board cheap enough
//! to step every frame.

use wasm_bindgen::prelude::*;

use crate::rng::Rng;

#[wasm_bindgen]
pub struct Life {
    width: usize,
    height: usize,
    cells: Vec<u8>,
    next: Vec<u8>,
    /// Generations each cell has been continuously alive, saturating at 255.
    age: Vec<u8>,
    pixels: Vec<u8>,
    cell_px: usize,
    generation: u32,
    population: u32,
    rng: Rng,
}

#[wasm_bindgen]
impl Life {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize, cell_px: usize) -> Life {
        let n = width * height;
        let mut life = Life {
            width,
            height,
            cells: vec![0; n],
            next: vec![0; n],
            age: vec![0; n],
            pixels: vec![0; width * cell_px * height * cell_px * 4],
            cell_px,
            generation: 0,
            population: 0,
            rng: Rng::new(0x5EED_1234),
        };
        life.randomize(0.28, 42);
        life
    }

    pub fn randomize(&mut self, density: f32, seed: u32) {
        self.rng = Rng::new(seed as u64 | 1);
        for c in self.cells.iter_mut() {
            *c = 0;
        }
        for i in 0..self.cells.len() {
            self.cells[i] = (self.rng.unit() < density) as u8;
        }
        self.age.iter_mut().for_each(|a| *a = 0);
        self.generation = 0;
        self.recount();
    }

    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|c| *c = 0);
        self.age.iter_mut().for_each(|a| *a = 0);
        self.generation = 0;
        self.population = 0;
    }

    pub fn toggle(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            let i = y * self.width + x;
            self.cells[i] ^= 1;
            self.age[i] = 0;
            self.recount();
        }
    }

    /// Stamps a Gosper glider gun with its top-left corner at (x, y).
    pub fn stamp_gun(&mut self, x: usize, y: usize) {
        const GUN: [(usize, usize); 36] = [
            (24, 0), (22, 1), (24, 1), (12, 2), (13, 2), (20, 2), (21, 2), (34, 2), (35, 2),
            (11, 3), (15, 3), (20, 3), (21, 3), (34, 3), (35, 3), (0, 4), (1, 4), (10, 4),
            (16, 4), (20, 4), (21, 4), (0, 5), (1, 5), (10, 5), (14, 5), (16, 5), (17, 5),
            (22, 5), (24, 5), (10, 6), (16, 6), (24, 6), (11, 7), (15, 7), (12, 8), (13, 8),
        ];
        for (dx, dy) in GUN {
            let px = (x + dx) % self.width;
            let py = (y + dy) % self.height;
            self.cells[py * self.width + px] = 1;
        }
        self.recount();
    }

    pub fn tick(&mut self) {
        let w = self.width;
        let h = self.height;
        let mut population = 0_u32;

        for y in 0..h {
            let y_up = if y == 0 { h - 1 } else { y - 1 } * w;
            let y_dn = if y == h - 1 { 0 } else { y + 1 } * w;
            let y_mid = y * w;

            for x in 0..w {
                let x_lf = if x == 0 { w - 1 } else { x - 1 };
                let x_rt = if x == w - 1 { 0 } else { x + 1 };

                let n = self.cells[y_up + x_lf]
                    + self.cells[y_up + x]
                    + self.cells[y_up + x_rt]
                    + self.cells[y_mid + x_lf]
                    + self.cells[y_mid + x_rt]
                    + self.cells[y_dn + x_lf]
                    + self.cells[y_dn + x]
                    + self.cells[y_dn + x_rt];

                let i = y_mid + x;
                let alive = self.cells[i] == 1;
                let born = n == 3 || (alive && n == 2);
                self.next[i] = born as u8;
                if born {
                    population += 1;
                    self.age[i] = if alive { self.age[i].saturating_add(1) } else { 0 };
                } else {
                    self.age[i] = 0;
                }
            }
        }

        std::mem::swap(&mut self.cells, &mut self.next);
        self.generation += 1;
        self.population = population;
    }

    /// Paints the board. Fresh cells read amber, long-lived ones settle into violet.
    pub fn render(&mut self) {
        let px = self.cell_px;
        let row_bytes = self.width * px * 4;

        for y in 0..self.height {
            for x in 0..self.width {
                let i = y * self.width + x;
                let (r, g, b) = if self.cells[i] == 0 {
                    (14, 16, 32)
                } else {
                    let t = (self.age[i] as f32 / 24.0).min(1.0);
                    (
                        (240.0 - 139.0 * t) as u8,
                        (180.0 - 101.0 * t) as u8,
                        (41.0 + 199.0 * t) as u8,
                    )
                };

                for sy in 0..px {
                    let mut o = (y * px + sy) * row_bytes + x * px * 4;
                    for _ in 0..px {
                        self.pixels[o] = r;
                        self.pixels[o + 1] = g;
                        self.pixels[o + 2] = b;
                        self.pixels[o + 3] = 255;
                        o += 4;
                    }
                }
            }
        }
    }

    fn recount(&mut self) {
        self.population = self.cells.iter().map(|&c| c as u32).sum();
    }

    pub fn pixels_ptr(&self) -> *const u8 {
        self.pixels.as_ptr()
    }
    pub fn pixels_len(&self) -> usize {
        self.pixels.len()
    }
    pub fn cells_len(&self) -> usize {
        self.cells.len()
    }
    pub fn generation(&self) -> u32 {
        self.generation
    }
    pub fn population(&self) -> u32 {
        self.population
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
}
