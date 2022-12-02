// Copyright(c) 2022 Hansen Audio.

use crate::{AudioFrame, NUM_CHANNELS};

#[derive(Clone, Copy)]
pub enum OnePoleType {
    LP,
    HP,
}

#[derive(Clone, Copy)]
pub struct OnePole {
    a: f32,
    cutoff: f32,
    filter_type: OnePoleType,
    omega: f32,
    sample_rate_recip: f32,
    x_1: [f32; NUM_CHANNELS],
    y_1: [f32; NUM_CHANNELS],
}

impl OnePole {
    const NUM_STEREO_CHANNELS: usize = 2;
    const PI_2: f32 = 2. * std::f32::consts::PI;

    pub fn new() -> Self {
        Self {
            a: 0.,
            cutoff: 0.,
            filter_type: OnePoleType::LP,
            omega: 1000. * Self::PI_2,
            sample_rate_recip: 1.,
            x_1: [0.; NUM_CHANNELS],
            y_1: [0.; NUM_CHANNELS],
        }
    }

    pub fn process_mono(&mut self, input: f32) -> f32 {
        const L_CH: usize = 0;
        let output = match self.filter_type {
            OnePoleType::LP => self.a * input + (1. - self.a) * self.y_1[L_CH],
            OnePoleType::HP => self.a * (self.y_1[L_CH] + input - self.x_1[L_CH]),
        };

        self.x_1[L_CH] = input;
        self.y_1[L_CH] = output;
        output
    }

    pub fn process(&mut self, ins: &AudioFrame, outs: &mut AudioFrame) {
        //Process only two channels for now
        self.process_stereo(ins, outs);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate_recip = sample_rate.recip();
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.cutoff = frequency;
        self.omega = Self::PI_2 * frequency;

        self.recalc_coeff();
    }

    pub fn reset(&mut self) {
        self.y_1 = [0.; NUM_CHANNELS];
        self.x_1 = [0.; NUM_CHANNELS];
    }

    pub fn set_filter_type(&mut self, filter_type: OnePoleType) {
        self.filter_type = filter_type;
        self.recalc_coeff();
    }

    fn process_stereo(&mut self, ins: &AudioFrame, outs: &mut AudioFrame) {
        for i in 0..Self::NUM_STEREO_CHANNELS {
            outs[i] = match self.filter_type {
                OnePoleType::LP => self.a * ins[i] + (1. - self.a) * self.y_1[i],
                OnePoleType::HP => self.a * (self.y_1[i] + ins[i] - self.x_1[i]),
            };

            self.x_1[i] = ins[i];
            self.y_1[i] = outs[i];
        }
    }

    fn _process_all_channels(&mut self, ins: &AudioFrame, outs: &mut AudioFrame) {
        for (i, el) in outs.iter_mut().enumerate() {
            match self.filter_type {
                OnePoleType::LP => *el = self.a * ins[i] + (1. - self.a) * self.y_1[i],
                OnePoleType::HP => *el = self.a * (self.y_1[i] + ins[i] - self.x_1[i]),
            }

            self.x_1[i] = ins[i];
            self.y_1[i] = *el;
        }
    }

    fn recalc_coeff(&mut self) {
        self.a = match self.filter_type {
            OnePoleType::LP => self.sample_rate_recip / (1. / self.omega + self.sample_rate_recip),
            OnePoleType::HP => (1. / self.omega) / (1. / self.omega + self.sample_rate_recip),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_line_multi() {
        let mut one_pole = OnePole::new();
        one_pole.set_sample_rate(44100.);
        one_pole.set_filter_type(OnePoleType::LP);
        one_pole.set_frequency(2000.);
        one_pole.reset();

        let ins: AudioFrame = [0.5, 0.25, 0.5, 0.75];
        let mut outs: AudioFrame = [0.; 4];

        for _ in 0..32 {
            one_pole.process(&ins, &mut outs);
        }

        assert_eq!(outs, [0.49983612, 0.24991806, 0.0, 0.0]);
    }

    #[test]
    fn test_delay_clone_copy() {
        let _filter_arr = [OnePole::new(); 2];
        let _filter_vec = vec![OnePole::new(); 2];
    }
}
