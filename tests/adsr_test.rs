// Copyright(c) 2021 Hansen Audio.

use dsp_tool_box_rs::modulation::adsr::Adsr;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn name() {
        let mut adsr = Adsr::new();
        adsr.set_attack(1.)
            .set_decay(1.)
            .set_sustain(1.)
            .set_release(1.);

        adsr.trigger();
        adsr.get_value(2.);
    }
}
