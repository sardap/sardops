use glam::Vec2;

#[derive(Copy, Clone, Default)]
pub struct Rect {
    // Note is the center
    pub pos: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub const fn new() -> Self {
        Self {
            pos: Vec2::new(0., 0.),
            size: Vec2::new(0., 0.),
        }
    }

    pub const fn new_center(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }

    pub const fn new_top_left(pos: Vec2, size: Vec2) -> Self {
        // Move pos to center
        let center_pos = Vec2::new(pos.x + size.x / 2., pos.y + size.y / 2.);

        Self {
            pos: center_pos,
            size,
        }
    }

    pub const fn pos_top_left(&self) -> Vec2 {
        let x_top_left = self.pos.x - self.size.x / 2.;
        let y_top_left = self.pos.y - self.size.y / 2.;
        Vec2::new(x_top_left, y_top_left)
    }

    pub fn random_point_inside(&self, rng: &mut fastrand::Rng) -> Vec2 {
        let top_left = self.pos_top_left();
        Vec2::new(
            rng.f32() * self.size.x + top_left.x,
            rng.f32() * self.size.y + top_left.y,
        )
    }

    pub fn point_inside(&self, pos: &Vec2) -> bool {
        pos.x > self.x() && pos.x < self.x2() && pos.y > self.y() && pos.y < self.y2()
    }

    pub fn overlapping(&self, other: &Self) -> bool {
        let half_self = self.size * 0.5;
        let half_other = other.size * 0.5;

        // delta between centers on each axis
        let delta = (self.pos - other.pos).abs();

        // overlap exists only if both axes satisfy the half-size sum condition
        delta.x <= (half_self.x + half_other.x) && delta.y <= (half_self.y + half_other.y)
    }

    pub const fn x(&self) -> f32 {
        self.pos.x - self.size.x / 2.
    }

    pub const fn y(&self) -> f32 {
        self.pos.y - self.size.y / 2.
    }

    pub const fn x2(&self) -> f32 {
        self.pos.x + self.size.x / 2.
    }

    pub const fn y2(&self) -> f32 {
        self.pos.y + self.size.y / 2.
    }

    #[allow(dead_code)]
    pub fn shrink(mut self, by: f32) -> Self {
        self.size.x = (self.size.x - by).max(0.);
        self.size.y = (self.size.y - by).max(0.);

        self
    }

    pub const fn grow(mut self, by: f32) -> Self {
        let mut pos = self.pos_top_left();
        self.size.x = self.size.x + by;
        self.size.y = self.size.y + by;
        pos.x -= by / 2.;
        pos.y -= by / 2.;
        self.pos = Vec2::new(pos.x + self.size.x / 2., pos.y + self.size.y / 2.);

        self
    }
}

pub fn vec2_distance(a: Vec2, b: Vec2) -> f32 {
    (a - b).length()
}

pub fn vec2_direction(a: Vec2, b: Vec2) -> Vec2 {
    (b - a).normalize()
}
