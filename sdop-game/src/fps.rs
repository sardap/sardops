use core::time::Duration;

const MAX_SAMPLES: usize = 100;

pub struct FPSCounter {
    tick_index: usize,
    tick_sum: i32,
    tick_list: [i32; MAX_SAMPLES],
    avg_tick: f32,
    frames: u32,
    frame_elapsed: Duration,
    last_fps: u32,
}

impl Default for FPSCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FPSCounter {
    pub fn new() -> Self {
        Self {
            tick_index: 0,
            tick_sum: 0,
            tick_list: [0; MAX_SAMPLES],
            avg_tick: 0.0,
            frames: 0,
            frame_elapsed: Duration::ZERO,
            last_fps: 0,
        }
    }

    pub fn update(&mut self, delta: Duration) {
        let new_tick = delta.as_millis() as i32;
        self.tick_sum -= self.tick_list[self.tick_index];
        self.tick_sum += new_tick;
        self.tick_list[self.tick_index] = new_tick;
        self.tick_index = (self.tick_index + 1) % self.tick_list.len();

        self.avg_tick = self.tick_sum as f32 / self.tick_list.len() as f32;

        self.frames += 1;
        self.frame_elapsed += delta;
        if self.frame_elapsed > Duration::from_secs(1) {
            self.last_fps = self.frames;
            self.frames = 0;
            self.frame_elapsed = Duration::ZERO;
        }
    }

    pub fn get_fps(&self) -> f32 {
        self.last_fps as f32
    }
}
