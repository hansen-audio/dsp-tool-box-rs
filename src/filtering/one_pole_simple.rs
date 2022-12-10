// Copyright(c) 2021 Hansen Audio.

use crate::{AudioFrame, NUM_CHANNELS};

#[derive(Debug, Clone)]
pub struct OnePoleSimple {
    a: f32,
    b: f32,
    z: f32,
}

impl OnePoleSimple {
    const FIVE_RECIP: f32 = 1. / 5.;

    pub fn new(a: f32) -> Self {
        Self {
            a,
            b: 1. - a,
            z: 0.,
        }
    }

    pub fn set_pole(&mut self, a: f32) {
        self.a = a;
        self.b = 1. - a;
    }

    pub fn set_tau(&mut self, tau: f32, sample_rate: f32) {
        self.set_pole(Self::tau_to_pole(tau, sample_rate));
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
        let result = -1. / ((tau * Self::FIVE_RECIP) * sample_rate);
        result.exp()
    }
}

#[derive(Debug, Clone)]
pub struct OnePoleSimpleMulti {
    a: f32,
    b: f32,
    z: AudioFrame,
}

impl OnePoleSimpleMulti {
    pub fn new(a: f32) -> Self {
        Self {
            a,
            b: 1. - a,
            z: [0.; NUM_CHANNELS],
        }
    }

    pub fn set_pole(&mut self, a: f32) {
        self.a = a;
        self.b = 1. - a;
    }

    pub fn set_tau(&mut self, tau: f32, sample_rate: f32) {
        self.set_pole(OnePoleSimple::tau_to_pole(tau, sample_rate));
    }

    pub fn process(&mut self, outputs: &mut AudioFrame) {
        use float_cmp::approx_eq;

        for i in 0..NUM_CHANNELS {
            let input = outputs[i];
            if !approx_eq!(f32, self.z[i], input) {
                outputs[i] = (input * self.b) + (self.z[i] * self.a);
                self.z[i] = outputs[i]
            }
        }
    }

    pub fn reset(&mut self, input: f32) {
        self.z.iter_mut().for_each(|item| *item = input);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NUM_ROUNDS: usize = 10;
    const RESULTS: [f32; NUM_ROUNDS] = [
        0.5, 0.75, 0.875, 0.9375, 0.96875, 0.984375, 0.9921875, 0.99609375, 0.9980469, 0.99902344,
    ];
    const POLE: f32 = 0.5;
    const INPUT: f32 = 1.;

    #[test]
    fn test_tau_to_pole() {
        let pole = OnePoleSimple::tau_to_pole(0.9, 48000.);
        assert_eq!(pole, 0.999884247);
    }

    #[test]
    fn test_one_pole() {
        let mut filter = OnePoleSimple::new(POLE);
        for item in RESULTS.into_iter() {
            let out = filter.process(INPUT);
            assert!(out == item);
            //println!("{:#?}", out);
        }
    }

    #[test]
    fn test_one_pole_simple_multi() {
        let mut ops_multi = OnePoleSimpleMulti::new(0.9);
        let inputs: AudioFrame = [0.; 4];
        let mut outputs: AudioFrame = [1., 0.75, 0.5, 0.25];

        ops_multi.process(&mut outputs);
        for _ in 0..31 {
            outputs.copy_from_slice(&inputs);
            ops_multi.process(&mut outputs);
            //assert_eq!(tmp, outputs);
        }

        assert_eq!(
            outputs,
            [0.0038152013, 0.0028614018, 0.0019076007, 0.00095380034]
        );
    }
}
