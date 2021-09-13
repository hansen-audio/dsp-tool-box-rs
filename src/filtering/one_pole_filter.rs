// Copyright(c) 2021 Hansen Audio.

use float_cmp::approx_eq;

use crate::RealType;

//#[repr(C)]
pub struct OnePoleContext {
    a: RealType,
    b: RealType,
    z: RealType,
}

impl OnePoleContext {
    pub fn new(a: RealType) -> Self {
        Self {
            a,
            b: 1. - a,
            z: 0.,
        }
    }

    pub fn update_pole(&mut self, a: RealType) {
        self.a = a;
        self.b = 1. - a;
    }

    pub fn process(&mut self, input: RealType) -> RealType {
        if approx_eq!(RealType, self.z, input) {
            return self.z;
        }

        self.z = (input * self.b) + (self.z * self.a);
        self.z
    }

    pub fn reset(&mut self, input: RealType) {
        self.z = input;
    }
}

pub fn tau_to_pole(tau: RealType, sample_rate: RealType) -> RealType {
    const RECIPROCAL_5: RealType = 1. / 5.;
    let result = -1. / ((tau * RECIPROCAL_5) * sample_rate);
    result.exp()
}

#[cfg(test)]
mod tests {
    use crate::filtering::one_pole_filter::OnePoleContext;

    use super::*;

    #[test]
    fn test_instantiation() {
        let mut c = OnePoleContext::new(0.);
        c.process(1.);
        c.update_pole(1.);
        c.reset(1.);
    }

    #[test]
    fn test_tau_to_pole() {
        let pole = tau_to_pole(0.9, 48000.);
        assert_eq!(pole, 0.999884247);
    }
}
