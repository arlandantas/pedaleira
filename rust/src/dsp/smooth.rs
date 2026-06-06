/// 1-pole exponential smoother. Ramps toward a target value over `time_ms`
/// milliseconds, eliminating audible clicks from abrupt parameter changes.
pub struct SmoothedParam {
    current: f32,
    target: f32,
    coeff: f32, // exp(-1 / (sample_rate * time_s))
}

impl SmoothedParam {
    /// `time_ms` is the 1-τ decay time (63% toward target). Use 5–10 ms.
    pub fn new(value: f32, sample_rate: f32, time_ms: f32) -> Self {
        let coeff = (-1.0_f32 / (sample_rate * time_ms / 1000.0)).exp();
        Self { current: value, target: value, coeff }
    }

    pub fn set(&mut self, target: f32) {
        self.target = target;
    }

    #[inline]
    pub fn next(&mut self) -> f32 {
        self.current = self.target + self.coeff * (self.current - self.target);
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_initial_value() {
        let mut p = SmoothedParam::new(0.5, 44100.0, 5.0);
        assert!((p.next() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn converges_to_target() {
        let mut p = SmoothedParam::new(0.0, 44100.0, 5.0);
        p.set(1.0);
        // Run for 50ms (2205 samples) — should be within 1% of target
        for _ in 0..2205 {
            p.next();
        }
        assert!((p.next() - 1.0).abs() < 0.01, "did not converge to target");
    }

    #[test]
    fn does_not_overshoot() {
        let mut p = SmoothedParam::new(0.0, 44100.0, 5.0);
        p.set(1.0);
        for _ in 0..10000 {
            let v = p.next();
            assert!(v <= 1.0 + 1e-6 && v >= -1e-6);
        }
    }
}
