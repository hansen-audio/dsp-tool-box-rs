// Copyright(c) 2021 Hansen Audio.

#[derive(Debug, Clone)]
pub enum SyncMode {
    FreeRunning,
    TempoSync,
    ProjectSync,
}

#[derive(Debug, Clone)]
pub struct Phase {
    free_run_factor: f32,
    mode: SyncMode,
    note_len: f32,
    project_time: f32,
    rate: f32,
    sample_rate_recip: f32,
    tempo: f32,
    tempo_synced_factor: f32,
}

impl Phase {
    pub fn new() -> Self {
        Self {
            free_run_factor: 0.,
            mode: SyncMode::ProjectSync,
            note_len: 1. / 32.,
            project_time: 0.,
            rate: 0.1,
            sample_rate_recip: 1. / 48000.,
            tempo_synced_factor: 0.,
            tempo: 120.,
        }
    }

    pub fn set_project_time(&mut self, value: f64) {
        const BEATS_IN_NOTE: f64 = 4.;
        let factor = (self.note_len as f64) * BEATS_IN_NOTE;

        self.project_time = (value % factor) as f32;
    }

    pub fn set_sync_mode(&mut self, value: SyncMode) {
        self.mode = value;
    }

    pub fn set_sample_rate(&mut self, value: f32) {
        self.sample_rate_recip = 1. / value;
        self.free_run_factor = Self::compute_free_running_factor(self.rate, self.sample_rate_recip);
        self.tempo_synced_factor =
            self.free_run_factor * Self::compute_tempo_synced_factor(self.tempo);
    }

    pub fn set_rate(&mut self, value: f32) {
        self.rate = value;
    }

    pub fn set_note_len(&mut self, value: f32) {
        self.note_len = value;
        let rate = Self::note_len_to_rate(value);
        self.set_rate(rate);
    }

    pub fn note_len(&self) -> f32 {
        self.note_len
    }

    pub fn set_tempo(&mut self, tempo_bpm: f32) {
        self.tempo = tempo_bpm;
        let factor = Self::compute_tempo_synced_factor(tempo_bpm);
        self.tempo_synced_factor = self.free_run_factor * factor;
    }

    pub fn advance(&self, value: &mut f32, num_samples: usize) -> bool {
        match self.mode {
            SyncMode::FreeRunning => {
                Self::update_free_running(value, num_samples, self.free_run_factor)
            }
            SyncMode::TempoSync => {
                Self::update_tempo_sync(value, num_samples, self.tempo_synced_factor)
            }
            SyncMode::ProjectSync => {
                let old_phase = *value;
                *value = Self::update_project_sync(self.project_time, self.rate);
                return *value < old_phase;
            }
        };

        Self::check_overflow(value)
    }

    pub fn advance_one_shot(&self, value: &mut f32, num_samples: usize) -> bool {
        match *value >= 1. {
            true => true,
            false => {
                let is_overflow = self.advance(value, num_samples);
                if is_overflow {
                    *value = 1.;
                }

                is_overflow
            }
        }
    }

    pub fn note_len_to_rate(value: f32) -> f32 {
        static BEATS_IN_NOTE_RECIP: f32 = 1. / 4.;
        assert!(value > 0.);
        (1. / value) * BEATS_IN_NOTE_RECIP
    }

    fn check_overflow(phase_value: &mut f32) -> bool {
        static PHASE_MAX: f32 = 1.;

        let overflow = *phase_value >= PHASE_MAX;
        if overflow {
            *phase_value %= PHASE_MAX;
        }

        overflow
    }

    fn update_free_running(phase: &mut f32, num_samples: usize, free_running_factor: f32) {
        *phase += free_running_factor * num_samples as f32;
    }

    fn update_tempo_sync(phase: &mut f32, num_samples: usize, tempo_synced_factor: f32) {
        *phase += num_samples as f32 * tempo_synced_factor;
    }

    fn update_project_sync(project_time: f32, rate: f32) -> f32 {
        project_time * rate
    }

    fn compute_free_running_factor(rate: f32, sample_rate_recip: f32) -> f32 {
        rate * sample_rate_recip
    }

    fn compute_tempo_synced_factor(tempo: f32) -> f32 {
        static SIXTY_SECS_RECIP: f32 = 1. / 60.;
        SIXTY_SECS_RECIP * tempo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let context = Phase::new();
        assert_eq!(context.tempo, 120.);
        assert_eq!(context.rate, 0.1);
    }

    #[test]
    fn test_advance() {
        let context = Phase::new();
        let mut value = 0.1;
        let did_overflow = context.advance(&mut value, 1);
        assert_eq!(did_overflow, true);
    }

    #[test]
    fn test_project_synced_overflow() {
        let mut phase_value = 0.;
        let mut context = Phase::new();
        context.set_sync_mode(SyncMode::ProjectSync);
        context.set_note_len(1.0);
        context.set_project_time(999999.9);

        let mut overflow = context.advance(&mut phase_value, 1);
        assert_eq!(overflow, false);

        context.set_project_time(4.0);
        overflow = context.advance(&mut phase_value, 1);
        assert_eq!(overflow, true);
    }

    #[test]
    #[ignore]
    fn test_debug_print() {
        let p = Phase::new();
        println!("{:#?}", p);
    }
}
