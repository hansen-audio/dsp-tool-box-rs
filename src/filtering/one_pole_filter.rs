// Copyright(c) 2021 Hansen Audio.

use crate::Real;

#[derive(Debug, Clone)]
pub struct OnePole {
    a: Real,
    b: Real,
    z: Real,
}

impl OnePole {
    pub fn new(a: Real) -> Self {
        Self {
            a,
            b: 1. - a,
            z: 0.,
        }
    }

    pub fn update_pole(&mut self, a: Real) {
        self.a = a;
        self.b = 1. - a;
    }

    pub fn process(&mut self, input: Real) -> Real {
        use float_cmp::approx_eq;

        if approx_eq!(Real, self.z, input) {
            return self.z;
        }

        self.z = (input * self.b) + (self.z * self.a);
        self.z
    }

    pub fn reset(&mut self, input: Real) {
        self.z = input;
    }

    pub fn tau_to_pole(tau: Real, sample_rate: Real) -> Real {
        const FIVE_RECIP: Real = 1. / 5.;
        let result = -1. / ((tau * FIVE_RECIP) * sample_rate);
        result.exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut c = OnePole::new(0.);
        c.process(1.);
        c.update_pole(1.);
        c.reset(1.);
    }

    #[test]
    fn test_tau_to_pole() {
        let pole = OnePole::tau_to_pole(0.9, 48000.);
        assert_eq!(pole, 0.999884247);
    }

    #[test]
    #[ignore]
    fn test_debug_print() {
        let filter = OnePole::new(0.);
        println!("{:#?}", filter);
    }
}
