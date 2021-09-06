// Copyright(c) 2021 Hansen Audio.

use float_cmp::{approx_eq, ApproxEq};

pub struct Context {
    a: f32,
    b: f32,
    z: f32,
}

impl Context {
    fn new(a: f32) -> Self {
        Self {
            a,
            b: 1. - a,
            z: 0.,
        }
    }

    fn update_pole(&mut self, a: f32) {
        self.a = a;
        self.b = 1. - a;
    }

    fn process(&mut self, input: f32) -> f32 {
        if approx_eq!(f32, self.z, input) {
            return self.z;
        }

        self.z = (input * self.b) + (self.z * self.a);
        self.z
    }

    fn reset(&mut self, input: f32) {
        self.z = input;
    }
}

fn tau_to_pole(tau: f32, sample_rate: f32) -> f32 {
    const RECIPROCAL_5: f32 = 1. / 5.;
    -1. / ((tau * RECIPROCAL_5) * sample_rate)
}

#[cfg(test)]
mod tests {
    use crate::filtering::one_pole_filter::Context;

    #[test]
    fn test_instantiation() {
        let mut c = Context::new(0.);
        c.process(1.);
    }
}
