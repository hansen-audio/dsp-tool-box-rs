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

    pub fn process(&mut self, input: &AudioFrame) -> AudioFrame {
        use float_cmp::approx_eq;

        for i in 0..NUM_CHANNELS {
            if !approx_eq!(f32, self.z[i], input[i]) {
                self.z[i] = (input[i] * self.b) + (self.z[i] * self.a);
            }
        }

        self.z
    }

    pub fn reset(&mut self, input: f32) {
        self.z.iter_mut().for_each(|item| *item = input);
    }
}
