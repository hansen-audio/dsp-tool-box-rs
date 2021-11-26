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
    const BEATS_IN_NOTE_RECIP: f32 = 1. / 4.;
    const BEATS_IN_NOTE: f64 = 4.;
    const PHASE_MAX: f32 = 1.;
    const SIXTY_SECS_RECIP: f32 = 1. / 60.;

    pub fn new() -> Self {
        Self {
            free_run_factor: 0.,
            note_len: 1. / 32.,
            project_time: 0.,
            rate: 0.1,
            sample_rate_recip: 1. / 48000.,
            sync_mode: SyncMode::ProjectSync,
            tempo_synced_factor: 0.,
            tempo: 120.,
        }
    }

    pub fn set_project_time(&mut self, project_time: f64) {
        // In order to keep the project_time as 'small' as
        // possible, the divider limits it to the range needed.
        let divider = (self.note_len as f64) * Self::BEATS_IN_NOTE;

        self.project_time = (project_time % divider) as f32;
    }

    pub fn set_sync_mode(&mut self, sync_mode: SyncMode) {
        self.sync_mode = sync_mode;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate_recip = 1. / sample_rate;
        self.free_run_factor = self.rate * self.sample_rate_recip;
        self.tempo_synced_factor = self.free_run_factor * (Self::SIXTY_SECS_RECIP * self.tempo);
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
        let factor = Self::SIXTY_SECS_RECIP * self.tempo;
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
        assert!(note_len > 0.);
        (1. / note_len) * Self::BEATS_IN_NOTE_RECIP
    }

    // private
    fn check_overflow(phase: &mut f32) -> bool {
        if *phase >= Self::PHASE_MAX {
            *phase %= Self::PHASE_MAX;
            return true;
        }

        false
    }
}
