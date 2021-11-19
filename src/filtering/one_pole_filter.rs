// Copyright(c) 2021 Hansen Audio.

#[derive(Debug, Clone)]
pub struct OnePole {
    a: f32,
    b: f32,
    z: f32,
}

impl OnePole {
    pub fn new(a: f32) -> Self {
        Self {
            a,
            b: 1. - a,
            z: 0.,
        }
    }

    pub fn update_pole(&mut self, a: f32) {
        self.a = a;
        self.b = 1. - a;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        use float_cmp::approx_eq;

        if approx_eq!(f32, self.z, input) {
            return self.z;
        }

        self.z = (input * self.b) + (self.z * self.a);
        self.z
    }

    pub fn reset(&mut self, input: f32) {
        self.z = input;
    }

    pub fn tau_to_pole(tau: f32, sample_rate: f32) -> f32 {
        const FIVE_RECIP: f32 = 1. / 5.;
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
