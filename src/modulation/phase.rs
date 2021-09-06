// Copyright(c) 2021 Hansen Audio.

use crate::RealType;

#[derive(Copy, Clone)]
pub enum SyncMode {
    FreeRunning,
    TempoSync,
    ProjectSync,
}

#[derive(Copy, Clone)]
pub struct Context {
    tempo: RealType,
    rate: RealType,
    sample_rate_recip: RealType,
    project_time: RealType,
    mode: SyncMode,
    free_running_factor: RealType,
    tempo_synced_factor: RealType,
    note_len: RealType,
}

static RECIPROCAL_BEATS_IN_NOTE: RealType = 1. / 4.;
static RECIPROCAL_60_SECONDS: RealType = 1. / 60.;
static PHASE_MAX: RealType = 1.;

fn check_overflow(phase_value: &mut RealType, phase_max: RealType) -> bool {
    let overflow = *phase_value >= phase_max;
    if overflow {
        *phase_value %= phase_max;
    }

    return overflow;
}

fn update_free_running(phase: &mut RealType, num_samples: i32, free_running_factor: RealType) {
    *phase += free_running_factor * num_samples as RealType;
}

fn update_tempo_sync(phase: &mut RealType, num_samples: i32, tempo_synced_factor: RealType) {
    *phase += num_samples as RealType * tempo_synced_factor;
}

fn normalize_phase(value: RealType) -> RealType {
    return value - value.floor();
}

fn update_project_sync(project_time: RealType, rate: RealType) -> RealType {
    return normalize_phase(project_time * rate);
}

fn compute_free_running_factor(rate: RealType, sample_rate_recip: RealType) -> RealType {
    return rate * sample_rate_recip;
}

fn compute_tempo_synced_factor(sixty_seconds_recip: RealType, tempo: RealType) -> RealType {
    return sixty_seconds_recip * tempo;
}

#[no_mangle]
pub extern "C" fn note_length_to_rate(value: RealType) -> RealType {
    assert!(value > 0.);
    return (1. / value) * RECIPROCAL_BEATS_IN_NOTE;
}

impl Context {
    fn set_project_time(&mut self, value: RealType) {
        self.project_time = value;
    }

    fn create() -> Self {
        Self {
            tempo: 120.,
            rate: 0.1,
            sample_rate_recip: 1. / 48000.,
            project_time: 0.,
            mode: SyncMode::ProjectSync,
            free_running_factor: 0.,
            tempo_synced_factor: 0.,
            note_len: 1. / 32.,
        }
    }

    fn set_sync_mode(&mut self, value: SyncMode) {
        self.mode = value;
    }

    fn set_sample_rate(&mut self, value: RealType) {
        self.sample_rate_recip = 1. / value;
        self.free_running_factor = compute_free_running_factor(self.rate, self.sample_rate_recip);
        self.tempo_synced_factor = self.free_running_factor
            * compute_tempo_synced_factor(RECIPROCAL_60_SECONDS, self.tempo);
    }

    fn set_rate(&mut self, value: RealType) {
        self.rate = value;
    }

    fn set_note_len(&mut self, value: RealType) {
        self.note_len = value;
        let rate = note_length_to_rate(value);
        self.set_rate(rate);
    }

    fn advance(&self, value: &mut RealType, num_samples: i32) -> bool {
        match self.mode {
            SyncMode::FreeRunning => {
                update_free_running(value, num_samples, self.free_running_factor)
            }
            SyncMode::TempoSync => update_tempo_sync(value, num_samples, self.tempo_synced_factor),
            SyncMode::ProjectSync => {
                let old_phase = *value;
                *value = update_project_sync(self.project_time, self.rate);
                return *value < old_phase;
            }
        };

        check_overflow(value, PHASE_MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let context = Context::create();
        assert_eq!(context.tempo, 120.);
        assert_eq!(context.rate, 0.1);
    }

    #[test]
    fn test_advance() {
        let context = Context::create();
        let mut value = 0.1;
        let did_overflow = context.advance(&mut value, 1);
        assert_eq!(did_overflow, true);
    }

    #[test]
    fn test_project_synced_overflow() {
        let mut phase_value = 0.;
        let mut context = Context::create();
        context.set_sync_mode(SyncMode::ProjectSync);
        context.set_note_len(1.0);
        //set_project_time(&mut cx, 3.9);
        context.set_project_time(3.9);
        let mut overflow = context.advance(&mut phase_value, 1);
        assert_eq!(overflow, false);

        context.set_project_time(4.0);
        overflow = context.advance(&mut phase_value, 1);
        assert_eq!(overflow, true);
    }
}
