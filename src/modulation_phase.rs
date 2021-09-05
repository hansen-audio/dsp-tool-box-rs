// Copyright(c) 2021 Hansen Audio.

#[repr(C)] // give this struct/union/enum the same layout and ABI C would
#[derive(Copy, Clone)]
pub enum SyncMode {
    FreeRunning,
    TempoSync,
    ProjectSync,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Context {
    tempo: f32,
    rate: f32,
    sample_rate_recip: f32,
    project_time: f32,
    mode: SyncMode,
    free_running_factor: f32,
    tempo_synced_factor: f32,
    note_len: f32,
}

static RECIPROCAL_BEATS_IN_NOTE: f32 = 1. / 4.;
static RECIPROCAL_60_SECONDS: f32 = 1. / 60.;
static PHASE_MAX: f32 = 1.;

fn check_overflow(phase_value: &mut f32, phase_max: f32) -> bool {
    let overflow = *phase_value >= phase_max;
    if overflow {
        *phase_value %= phase_max;
    }

    return overflow;
}

fn update_free_running(phase: &mut f32, num_samples: i32, free_running_factor: f32) {
    *phase += free_running_factor * num_samples as f32;
}

fn update_tempo_sync(phase: &mut f32, num_samples: i32, tempo_synced_factor: f32) {
    *phase += num_samples as f32 * tempo_synced_factor;
}

fn normalize_phase(value: f32) -> f32 {
    return value - value.floor();
}

fn update_project_sync(project_time: f32, rate: f32) -> f32 {
    return normalize_phase(project_time * rate);
}

fn compute_free_running_factor(rate: f32, sample_rate_recip: f32) -> f32 {
    return rate * sample_rate_recip;
}

fn compute_tempo_synced_factor(sixty_seconds_recip: f32, tempo: f32) -> f32 {
    return sixty_seconds_recip * tempo;
}

fn note_length_to_rate(value: f32) -> f32 {
    assert!(value > 0.);
    return (1. / value) * RECIPROCAL_BEATS_IN_NOTE;
}

impl Context {
    fn set_project_time(&mut self, value: f32) {
        (*self).project_time = value;
    }

    fn create() -> Context {
        let phase = Context {
            tempo: 120.,
            rate: 0.1,
            sample_rate_recip: 1. / 48000.,
            project_time: 0.,
            mode: SyncMode::ProjectSync,
            free_running_factor: 0.,
            tempo_synced_factor: 0.,
            note_len: 1. / 32.,
        };

        return phase;
    }

    fn set_sync_mode(&mut self, value: SyncMode) {
        self.mode = value;
    }

    fn set_sample_rate(&mut self, value: f32) {
        self.sample_rate_recip = 1. / value;
        self.free_running_factor = compute_free_running_factor(self.rate, self.sample_rate_recip);
        self.tempo_synced_factor = self.free_running_factor
            * compute_tempo_synced_factor(RECIPROCAL_60_SECONDS, self.tempo);
    }

    fn set_rate(&mut self, value: f32) {
        self.rate = value;
    }

    fn set_note_len(&mut self, value: f32) {
        self.note_len = value;
        let rate = note_length_to_rate(value);
        self.set_rate(rate);
    }

    fn advance(&self, value: &mut f32, num_samples: i32) -> bool {
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

        return check_overflow(value, PHASE_MAX);
    }
}

// C bindings
#[no_mangle]
pub extern "C" fn set_project_time(context: &mut Context, value: f32) {
    context.set_project_time(value);
}

#[no_mangle]
pub extern "C" fn create() -> Context {
    Context::create()
}

#[no_mangle]
pub extern "C" fn set_sync_mode(context: &mut Context, value: SyncMode) {
    context.set_sync_mode(value);
}

#[no_mangle]
pub extern "C" fn set_sample_rate(context: &mut Context, value: f32) {
    context.set_sample_rate(value);
}

#[no_mangle]
pub extern "C" fn set_rate(context: &mut Context, value: f32) {
    context.set_rate(value);
}

#[no_mangle]
pub extern "C" fn set_note_len(context: &mut Context, value: f32) {
    context.set_note_len(value);
}

#[no_mangle]
pub extern "C" fn advance(context: &Context, value: &mut f32, num_samples: i32) -> bool {
    context.advance(value, num_samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let phase = Context::create();
        assert_eq!(phase.tempo, 120.);
        assert_eq!(phase.rate, 0.1);
    }

    #[test]
    fn test_advance() {
        let phase = Context::create();
        let mut value = 0.1;
        let did_overflow = phase.advance(&mut value, 1);
        assert_eq!(did_overflow, true);
    }

    #[test]
    fn test_project_synced_overflow() {
        let mut phase_value = 0.;
        let mut cx = Context::create();
        cx.set_sync_mode(SyncMode::ProjectSync);
        cx.set_note_len(1.0);
        //set_project_time(&mut cx, 3.9);
        cx.set_project_time(3.9);
        let mut overflow = cx.advance(&mut phase_value, 1);
        assert_eq!(overflow, false);

        cx.set_project_time(4.0);
        overflow = cx.advance(&mut phase_value, 1);
        assert_eq!(overflow, true);
    }
}
