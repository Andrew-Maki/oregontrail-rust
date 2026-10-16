//! A tiny self-contained pseudo-random number generator.
//!
//! The game is deliberately dependency-free, so instead of pulling in `rand`
//! we use a `xorshift64*` generator. It is far more than good enough for
//! simulating dysentery.

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }

    /// Seed from the wall clock and the process id.
    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x1234_5678_9ABC_DEF0);
        let pid = u64::from(std::process::id());
        let mut rng = Rng::new(nanos ^ (pid << 31) ^ 0xA5A5_5A5A_DEAD_BEEF);
        // Churn a little so nearby seeds diverge immediately.
        for _ in 0..8 {
            rng.next_u64();
        }
        rng
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform float in `[0.0, 1.0)`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform integer in `0..n`.
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % u64::from(n)) as u32
    }

    /// Uniform integer in `lo..=hi`.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + self.below((hi - lo + 1) as u32) as i32
    }

    /// Uniform float in `lo..hi`.
    pub fn range_f(&mut self, lo: f64, hi: f64) -> f64 {
        if hi <= lo {
            return lo;
        }
        lo + self.unit() * (hi - lo)
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.unit() < p
    }

    /// Pick a random element from a non-empty slice.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        debug_assert!(!items.is_empty(), "pick() on an empty slice");
        let i = self.below(items.len() as u32) as usize;
        &items[i]
    }

    /// Average of two rolls, which clusters results toward the middle.
    pub fn range_centered(&mut self, lo: i32, hi: i32) -> i32 {
        (self.range(lo, hi) + self.range(lo, hi)) / 2
    }
}
