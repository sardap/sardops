// I really don't understand math
pub const fn norm_tau(mut a: f32) -> f32 {
    let tau = core::f32::consts::TAU;
    if a >= tau {
        a -= tau;
    }
    if a < 0.0 {
        a += tau;
    }
    a
}
