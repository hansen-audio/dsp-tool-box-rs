#[repr(C)]
#[derive(Copy, Clone)]
pub enum SyncMode {
    FreeRunning,
    TempoSync,
    ProjectSync,
}

#[derive(Copy, Clone)]
pub struct Phase {
    tempo: f32,
    rate: f32,
    sample_rate_recip: f32,
    project_time: f32,
    mode: SyncMode,
    free_running_factor: f32,
    tempo_synced_factor: f32,
    note_len: f32,
} //

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

#[no_mangle]
pub extern "C" fn create() -> Phase {
    let phase = Phase {
        tempo: 120.,
        rate: 0.1,
        sample_rate_recip: 1. / 48000.,
        project_time: 0.,
        mode: SyncMode::ProjectSync,
        free_running_factor: 0.,
        tempo_synced_factor: 0.,
        note_len: 1. / 32.,
    };

    phase
}

#[no_mangle]
pub extern "C" fn advance(phase: Phase, value: &mut f32, num_samples: i32) -> bool {
    match phase.mode {
        SyncMode::FreeRunning => update_free_running(value, num_samples, phase.free_running_factor),
        SyncMode::TempoSync => update_tempo_sync(value, num_samples, phase.tempo_synced_factor),
        SyncMode::ProjectSync => {
            let old_phase = *value;
            *value = update_project_sync(phase.project_time, phase.rate);
            return *value < old_phase;
        }
    };

    return check_overflow(value, PHASE_MAX);
}

#[no_mangle]
pub fn note_length_to_rate(value: f32) -> f32 {
    assert!(value > 0.);
    return (1. / value) * RECIPROCAL_BEATS_IN_NOTE;
}

#[no_mangle]
pub extern "C" fn set_rate(cx: &mut Phase, value: f32) {
    cx.rate = value;
}

#[no_mangle]
pub extern "C" fn set_note_len(cx: &mut Phase, value: f32) {
    cx.note_len = value;
    let rate = note_length_to_rate(value);
    set_rate(cx, rate);
}

#[no_mangle]
pub extern "C" fn set_project_time(cx: &mut Phase, value: f32) {
    cx.project_time = value;
}

#[no_mangle]
pub extern "C" fn set_sync_mode(cx: &mut Phase, value: SyncMode) {
    cx.mode = value;
}

#[no_mangle]
pub extern "C" fn set_sample_rate(cx: &mut Phase, value: f32) {
    cx.sample_rate_recip = 1. / value;
    cx.free_running_factor = compute_free_running_factor(cx.rate, cx.sample_rate_recip);
    cx.tempo_synced_factor =
        cx.free_running_factor * compute_tempo_synced_factor(RECIPROCAL_60_SECONDS, cx.tempo);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let phase = create();
        assert_eq!(phase.tempo, 120.);
        assert_eq!(phase.rate, 0.1);
    }

    #[test]
    fn test_advance() {
        let phase = create();
        let mut value = 0.1;
        let did_overflow = advance(phase, &mut value, 1);
        assert_eq!(did_overflow, true);
    }

    #[test]
    fn test_project_synced_overflow() {
        let mut phase_value = 0.;
        let mut cx = create();
        set_sync_mode(&mut cx, SyncMode::ProjectSync);
        set_note_len(&mut cx, 1.0);
        set_project_time(&mut cx, 3.9);
        let mut overflow = advance(cx, &mut phase_value, 1);
        assert_eq!(overflow, false);

        set_project_time(&mut cx, 4.0);
        overflow = advance(cx, &mut phase_value, 1);
        assert_eq!(overflow, true);
    }
}
