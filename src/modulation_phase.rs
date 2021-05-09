#[repr(C)]
pub struct Phase {
    value: f32,
    tempo: f32,
    rate: f32,
}

#[no_mangle]
pub extern "C" fn create() -> Phase {
    let phase = Phase {
        value: 0.1,
        tempo: 120.,
        rate: 0.1,
    };

    phase
}

#[no_mangle]
pub extern "C" fn advance(phase: &Phase, value: f32) -> f32 {
    return value + phase.value;
}
