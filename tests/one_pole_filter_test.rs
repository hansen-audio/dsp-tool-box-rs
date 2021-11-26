// Copyright(c) 2021 Hansen Audio.

use dsp_tool_box_rs::filtering::one_pole_filter::OnePole;

#[cfg(test)]
mod tests {

    use super::*;

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
