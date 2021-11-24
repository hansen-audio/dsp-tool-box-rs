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
    sync_mode: SyncMode,
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
            sync_mode: SyncMode::ProjectSync,
            note_len: 1. / 32.,
            project_time: 0.,
            rate: 0.1,
            sample_rate_recip: 1. / 48000.,
            tempo_synced_factor: 0.,
            tempo: 120.,
        }
    }

    pub fn set_project_time(&mut self, project_time: f64) {
        const BEATS_IN_NOTE: f64 = 4.;
        let factor = (self.note_len as f64) * BEATS_IN_NOTE;

        self.project_time = (project_time % factor) as f32;
    }

    pub fn set_sync_mode(&mut self, sync_mode: SyncMode) {
        self.sync_mode = sync_mode;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate_recip = 1. / sample_rate;
        self.free_run_factor = self.rate * self.sample_rate_recip;
        self.tempo_synced_factor =
            self.free_run_factor * Self::compute_tempo_synced_factor(self.tempo);
    }

    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
    }

    pub fn set_note_len(&mut self, note_len: f32) {
        self.note_len = note_len;
        let rate = Self::note_len_to_rate(note_len);
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

    pub fn advance(&self, phase: &mut f32, num_samples: usize) -> bool {
        match self.sync_mode {
            SyncMode::FreeRunning => {
                *phase += self.free_run_factor * num_samples as f32;
            }
            SyncMode::TempoSync => *phase += num_samples as f32 * self.tempo_synced_factor,
            SyncMode::ProjectSync => {
                let old = *phase;
                *phase = self.project_time * self.rate;
                return *phase < old;
            }
        };

        Self::check_overflow(phase)
    }

    pub fn advance_one_shot(&self, phase: &mut f32, num_samples: usize) -> bool {
        match *phase >= 1. {
            true => true,
            false => {
                let is_overflow = self.advance(phase, num_samples);
                if is_overflow {
                    *phase = 1.;
                }

                is_overflow
            }
        }
    }

    pub fn note_len_to_rate(note_len: f32) -> f32 {
        static BEATS_IN_NOTE_RECIP: f32 = 1. / 4.;
        assert!(note_len > 0.);
        (1. / note_len) * BEATS_IN_NOTE_RECIP
    }

    fn check_overflow(phase: &mut f32) -> bool {
        static PHASE_MAX: f32 = 1.;

        let overflow = *phase >= PHASE_MAX;
        if overflow {
            *phase %= PHASE_MAX;
        }

        overflow
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
