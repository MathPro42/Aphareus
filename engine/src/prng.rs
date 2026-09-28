/// A xorshift64* generator.
pub struct Prng {
    state: u64,
}

impl Prng {
    /// Creates a generator from a non-zero `seed`.
    pub const fn new(seed: u64) -> Prng {
        assert!(seed != 0, "xorshift needs a non-zero seed");
        Prng { state: seed }
    }

    /// The next pseudo-random 64-bit value.
    pub const fn next_u64(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}
