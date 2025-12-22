use glam::{I16Vec2, IVec2, Vec2};

include!(concat!(env!("OUT_DIR"), "/dist_geo.rs"));

// impl Into<RectVec2> for RectIVec2 {
//     fn into(self) -> RectVec2 {
//         RectVec2::new_center(
//             Vec2::new(self.x() as f32, self.y() as f32),
//             Vec2::new(self.size.x as f32, self.size.y as f32),
//         )
//     }
// }

pub fn vec2_distance(a: Vec2, b: Vec2) -> f32 {
    (a - b).length()
}

pub fn vec2_direction(a: Vec2, b: Vec2) -> Vec2 {
    (b - a).normalize()
}
