use glam::{I16Vec2, IVec2, Vec2};

include!(concat!(env!("OUT_DIR"), "/dist_geo.rs"));

pub fn vec2_distance(a: Vec2, b: Vec2) -> f32 {
    (a - b).length()
}

pub fn vec2_direction(a: Vec2, b: Vec2) -> Vec2 {
    (b - a).normalize()
}

// It's annoying how you can implment into for forigin types
pub fn ivec_to_vec2(val: IVec2) -> Vec2 {
    Vec2::new(val.x as f32, val.y as f32)
}

pub fn vec2_to_ivec2(val: Vec2) -> IVec2 {
    IVec2::new(val.x as i32, val.y as i32)
}
