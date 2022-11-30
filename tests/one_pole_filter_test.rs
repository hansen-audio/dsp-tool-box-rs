// Copyright(c) 2021 Hansen Audio.

use dsp_tool_box_rs::filtering::one_pole_simple::OnePoleSimple;
use dsp_tool_box_rs::filtering::one_pole_simple::OnePoleSimpleMulti;

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
    fn test_one_pole_multi() {
        let mut filter = OnePoleSimpleMulti::new(POLE);
        for item in RESULTS.into_iter() {
            let out = filter.process(&[INPUT, INPUT, INPUT, INPUT]);
            assert!(out == [item, item, item, item]);
            //println!("{:#?}", out);
        }
    }
}
