// Copyright(c) 2021 Hansen Audio.

use crate::Real;

#[derive(Debug, Copy, Clone)]
pub enum SyncMode {
    FreeRunning,
    TempoSync,
    ProjectSync,
}

#[derive(Debug, Copy, Clone)]
pub struct Phase {
    free_run_factor: Real,
    mode: SyncMode,
    note_len: Real,
    project_time: Real,
    rate: Real,
    sample_rate_recip: Real,
    tempo: Real,
    tempo_synced_factor: Real,
}

static BEATS_IN_NOTE_RECIP: Real = 1. / 4.;
static SIXTY_SECONDS_RECIP: Real = 1. / 60.;
static PHASE_MAX: Real = 1.;

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

    pub fn set_project_time(&mut self, value: Real) {
        self.project_time = value;
    }

    pub fn set_sync_mode(&mut self, value: SyncMode) {
        self.mode = value;
    }

    pub fn set_sample_rate(&mut self, value: Real) {
        self.sample_rate_recip = 1. / value;
        self.free_run_factor = compute_free_running_factor(self.rate, self.sample_rate_recip);
        self.tempo_synced_factor =
            self.free_run_factor * compute_tempo_synced_factor(SIXTY_SECONDS_RECIP, self.tempo);
    }

    pub fn set_rate(&mut self, value: Real) {
        self.rate = value;
    }

    pub fn set_note_len(&mut self, value: Real) {
        self.note_len = value;
        let rate = note_length_to_rate(value);
        self.set_rate(rate);
    }

    pub fn get_note_len(&self) -> Real {
        self.note_len
    }

    pub fn set_tempo(&mut self, tempo_bpm: Real) {
        self.tempo = tempo_bpm;
        let factor = compute_tempo_synced_factor(SIXTY_SECONDS_RECIP, tempo_bpm);
        self.tempo_synced_factor = self.free_run_factor * factor;
    }

    pub fn advance(&self, value: &mut Real, num_samples: usize) -> bool {
        match self.mode {
            SyncMode::FreeRunning => update_free_running(value, num_samples, self.free_run_factor),
            SyncMode::TempoSync => update_tempo_sync(value, num_samples, self.tempo_synced_factor),
            SyncMode::ProjectSync => {
                let old_phase = *value;
                *value = update_project_sync(self.project_time, self.rate);
                return *value < old_phase;
            }
        };

        check_overflow(value, PHASE_MAX)
    }

    pub fn advance_one_shot(&self, value: &mut Real, num_samples: usize) -> bool {
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
}

fn check_overflow(phase_value: &mut Real, phase_max: Real) -> bool {
    let overflow = *phase_value >= phase_max;
    if overflow {
        *phase_value %= phase_max;
    }

    overflow
}

fn update_free_running(phase: &mut Real, num_samples: usize, free_running_factor: Real) {
    *phase += free_running_factor * num_samples as Real;
}

fn update_tempo_sync(phase: &mut Real, num_samples: usize, tempo_synced_factor: Real) {
    *phase += num_samples as Real * tempo_synced_factor;
}

fn normalize_phase(value: Real) -> Real {
    value - value.floor()
}

fn update_project_sync(project_time: Real, rate: Real) -> Real {
    normalize_phase(project_time * rate)
}

fn compute_free_running_factor(rate: Real, sample_rate_recip: Real) -> Real {
    rate * sample_rate_recip
}

fn compute_tempo_synced_factor(sixty_seconds_recip: Real, tempo: Real) -> Real {
    sixty_seconds_recip * tempo
}

pub fn note_length_to_rate(value: Real) -> Real {
    assert!(value > 0.);
    (1. / value) * BEATS_IN_NOTE_RECIP
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
        context.set_project_time(3.9);

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
