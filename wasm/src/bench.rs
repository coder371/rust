//! Workloads that exist in Rust, Go and JavaScript so the three can be timed
//! against each other on identical input.
//!
//! Each one returns a checksum. The caller compares checksums across the three
//! implementations, which both proves they did the same work and stops any
//! optimiser from deleting a loop whose result is never read.
//!
//! The eight tasks are chosen to stress different things: floating-point
//! throughput, memory writes, a pure scalar loop, memory bandwidth, byte
//! throughput, recursion, and allocation churn.

use wasm_bindgen::prelude::*;

/// Naive dense matrix multiply, O(n^3), on deterministic synthetic input.
#[wasm_bindgen]
pub fn bench_matmul(n: usize) -> f64 {
    let mut a = vec![0.0_f64; n * n];
    let mut b = vec![0.0_f64; n * n];
    for i in 0..n * n {
        a[i] = ((i % 17) as f64) * 0.125 - 1.0;
        b[i] = ((i % 23) as f64) * 0.0625 - 0.5;
    }

    let mut c = vec![0.0_f64; n * n];
    for i in 0..n {
        for k in 0..n {
            let aik = a[i * n + k];
            if aik == 0.0 {
                continue;
            }
            for j in 0..n {
                c[i * n + j] += aik * b[k * n + j];
            }
        }
    }

    c.iter().sum()
}

/// Sieve of Eratosthenes; returns the count of primes below `limit`.
#[wasm_bindgen]
pub fn bench_sieve(limit: usize) -> f64 {
    if limit < 2 {
        return 0.0;
    }
    let mut composite = vec![false; limit];
    let mut count = 0_u32;
    let mut i = 2;
    while i < limit {
        if !composite[i] {
            count += 1;
            // `usize` is 32 bits on wasm32, and `i * i` wraps for any prime
            // above 65535 — which silently corrupts the count. Skip instead:
            // those primes have no multiples below the limit to mark anyway.
            if let Some(square) = i.checked_mul(i) {
                let mut j = square;
                while j < limit {
                    composite[j] = true;
                    j += i;
                }
            }
        }
        i += 1;
    }
    count as f64
}

/// Escape-time iteration over a fixed grid — the same inner loop the fractal
/// view runs, isolated so it can be measured on its own.
#[wasm_bindgen]
pub fn bench_escape(side: usize, max_iter: u32) -> f64 {
    let mut total = 0_u64;
    for y in 0..side {
        let ci = -1.25 + 2.5 * (y as f64 / side as f64);
        for x in 0..side {
            let cr = -2.0 + 3.0 * (x as f64 / side as f64);
            let mut zr = 0.0_f64;
            let mut zi = 0.0_f64;
            let mut i = 0_u32;
            while i < max_iter && zr * zr + zi * zi <= 4.0 {
                let t = zr * zr - zi * zi + cr;
                zi = 2.0 * zr * zi + ci;
                zr = t;
                i += 1;
            }
            total += i as u64;
        }
    }
    total as f64
}

/// Integer mixing loop with a serial dependency — no vectorisation available
/// to any runtime, so it measures raw scalar throughput.
#[wasm_bindgen]
pub fn bench_mix(rounds: u32) -> f64 {
    let mut h: u32 = 0x811C_9DC5;
    for i in 0..rounds {
        h ^= i;
        h = h.wrapping_mul(0x0100_0193);
        h ^= h >> 15;
        h = h.wrapping_add(h << 7);
    }
    h as f64
}

/// Least-significant-digit radix sort, four 8-bit passes over `n` words.
///
/// Almost pure memory traffic: the scatter step writes to 256 cursors at once,
/// so it defeats the cache in a way none of the other tasks do.
#[wasm_bindgen]
pub fn bench_radix(n: usize) -> f64 {
    let mut src = vec![0_u32; n];
    let mut x: u32 = 0x9296_1E37;
    for slot in src.iter_mut() {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *slot = x;
    }

    let mut dst = vec![0_u32; n];
    for shift in [0_u32, 8, 16, 24] {
        let mut count = [0_u32; 256];
        for &v in src.iter() {
            count[((v >> shift) & 0xFF) as usize] += 1;
        }
        let mut sum = 0_u32;
        for slot in count.iter_mut() {
            let c = *slot;
            *slot = sum;
            sum += c;
        }
        for &v in src.iter() {
            let bucket = ((v >> shift) & 0xFF) as usize;
            dst[count[bucket] as usize] = v;
            count[bucket] += 1;
        }
        std::mem::swap(&mut src, &mut dst);
    }

    // Hashing the sorted run proves the ordering matches, not merely the set.
    let mut h: u32 = 0x811C_9DC5;
    for &v in src.iter() {
        h ^= v;
        h = h.wrapping_mul(0x0100_0193);
    }
    h as f64
}

/// Table-driven CRC-32 over a generated buffer, folded `rounds` times.
/// One dependent table lookup per byte: latency-bound, not throughput-bound.
#[wasm_bindgen]
pub fn bench_crc(len: usize, rounds: u32) -> f64 {
    let mut table = [0_u32; 256];
    for (i, slot) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *slot = c;
    }

    let mut data = vec![0_u8; len];
    for (i, slot) in data.iter_mut().enumerate() {
        *slot = ((i.wrapping_mul(167)) ^ (i >> 3)) as u8;
    }

    let mut crc: u32 = 0xFFFF_FFFF;
    for _ in 0..rounds {
        for &byte in data.iter() {
            crc = (crc >> 8) ^ table[((crc ^ byte as u32) & 0xFF) as usize];
        }
    }
    (crc ^ 0xFFFF_FFFF) as f64
}

/// Counts the solutions to the n-queens problem with a bitmask search.
/// Deep recursion over a branchy tree — it measures call overhead and branch
/// prediction rather than arithmetic.
#[wasm_bindgen]
pub fn bench_queens(n: u32) -> f64 {
    fn search(cols: u32, ld: u32, rd: u32, all: u32) -> u32 {
        if cols == all {
            return 1;
        }
        let mut count = 0;
        let mut open = !(cols | ld | rd) & all;
        while open != 0 {
            let bit = open & open.wrapping_neg();
            open -= bit;
            count += search(cols | bit, ((ld | bit) << 1) & all, (rd | bit) >> 1, all);
        }
        count
    }

    let all = (1_u32 << n) - 1;
    search(0, 0, 0, all) as f64
}

struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

fn build_tree(depth: u32) -> Node {
    if depth == 0 {
        Node { left: None, right: None }
    } else {
        Node {
            left: Some(Box::new(build_tree(depth - 1))),
            right: Some(Box::new(build_tree(depth - 1))),
        }
    }
}

fn check_tree(node: &Node) -> i64 {
    match (&node.left, &node.right) {
        (Some(l), Some(r)) => 1 + check_tree(l) + check_tree(r),
        _ => 1,
    }
}

/// Builds and walks many short-lived binary trees.
///
/// This is the only task dominated by allocation and reclamation, which is
/// where an owning allocator, a tracing collector and a JavaScript heap
/// behave least alike.
#[wasm_bindgen]
pub fn bench_trees(max_depth: u32) -> f64 {
    let mut total: i64 = 0;
    let mut depth = 4;
    while depth <= max_depth {
        let iterations = 1_u32 << (max_depth - depth + 4);
        for _ in 0..iterations {
            total += check_tree(&build_tree(depth));
        }
        depth += 2;
    }
    total as f64
}
