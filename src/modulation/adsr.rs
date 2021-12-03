// Copyright(c) 2021 Hansen Audio.

struct StageTime {
    seconds: f32,
    seconds_recip: f32,
}

impl StageTime {
    pub fn new(time: f32) -> Self {
        Self {
            seconds: time,
            seconds_recip: Self::calc_recip_from_time(time),
        }
    }

    pub fn set_time(&mut self, time: f32) {
        self.seconds = time;
        self.seconds_recip = Self::calc_recip_from_time(time);
    }

    fn calc_recip_from_time(time: f32) -> f32 {
        if time > 0. {
            1. / time
        } else {
            0.
        }
    }
}

enum Stage {
    BeforeTrigger,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Adsr {
    stage: Stage,
    attack: StageTime,
    decay: StageTime,
    sustain: f32,
    release: StageTime,
    gate_release_value: f32,
    ease_multiplicator: f32,
}

impl Adsr {
    const MIN_VALUE: f32 = 0.;
    const MAX_VALUE: f32 = 1.;

    pub fn new() -> Self {
        Self {
            stage: Stage::BeforeTrigger,
            attack: StageTime::new(1.),
            decay: StageTime::new(1.),
            sustain: Self::MAX_VALUE,
            release: StageTime::new(1.),
            gate_release_value: Self::MAX_VALUE,
            ease_multiplicator: calc_ease_virus_ti_multiplicator(),
        }
    }

    pub fn trigger(&mut self) {
        self.stage = Stage::Attack;
        self.gate_release_value = Self::MAX_VALUE;
    }

    pub fn on_release(&mut self) {
        self.stage = Stage::Release;
    }

    pub fn get_value(&mut self, time: f32) -> f32 {
        match self.stage {
            Stage::BeforeTrigger => 0.,
            Stage::Attack => self.attack(time),
            Stage::Decay => self.decay(time),
            Stage::Sustain => self.sustain(),
            Stage::Release => self.release(time),
        }
    }

    pub fn set_attack(&mut self, time: f32) -> &mut Self {
        self.attack.set_time(time);
        self
    }

    pub fn set_decay(&mut self, time: f32) -> &mut Self {
        self.decay.set_time(time);
        self
    }

    pub fn set_sustain(&mut self, level: f32) -> &mut Self {
        self.sustain = level;
        self
    }

    pub fn set_release(&mut self, time: f32) -> &mut Self {
        self.release.set_time(time);
        self
    }

    // private
    fn attack(&mut self, time: f32) -> f32 {
        self.stage = Stage::Attack;
        if time > self.attack.seconds {
            return self.decay(time);
        }

        self.gate_release_value = time * self.attack.seconds_recip;

        self.gate_release_value
    }

    fn decay(&mut self, _time: f32) -> f32 {
        self.stage = Stage::Decay;
        let time = _time - self.attack.seconds;
        if time > self.decay.seconds {
            return self.sustain();
        }

        let value = time * self.decay.seconds_recip;
        self.gate_release_value =
            (self.sustain - Self::MAX_VALUE) * self.shape(value) + Self::MAX_VALUE;

        self.gate_release_value
    }

    fn sustain(&mut self) -> f32 {
        self.stage = Stage::Sustain;
        self.gate_release_value = self.sustain;

        self.sustain
    }

    fn release(&mut self, time: f32) -> f32 {
        /*!
            The Fourth cycle is 'release' starting when key is released.
            It runs to 0 beginning from the value the envelope had when
            releasing the key. Often this is 'sustain' value.
        */
        self.stage = Stage::Release;
        if time > self.release.seconds {
            self.stage = Stage::BeforeTrigger;
            return Self::MIN_VALUE;
        }

        let value = time * self.release.seconds_recip;

        self.gate_release_value - self.shape(value) * self.gate_release_value
    }

    fn shape(&self, x: f32) -> f32 {
        ease_virus_ti(x, self.ease_multiplicator)
    }
}

fn calc_ease_virus_ti_multiplicator() -> f32 {
    /*
        float x = powf (0.5f, x * 14.f);
        float db = 20.f * log10f (x);
        --> ca. -96dB....sufficient!
    */
    let db_value = 96.;
    let factor = (10. as f32).powf(db_value / (20. as f32));
    factor.log2()
}

fn ease_virus_ti(x: f32, multiplicator: f32) -> f32 {
    (0.5 as f32).powf(x * multiplicator)
}
