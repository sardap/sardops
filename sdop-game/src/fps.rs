use core::time::Duration;

use crate::Timestamp;

pub struct FPSCounter {
    last_time: Timestamp,
    fps: f32,
    avg_fps: f32,
    frame_count: u32,
    elapsed: Duration,
}

impl FPSCounter {
    pub fn new(timestamp: Timestamp) -> Self {
        Self {
            last_time: timestamp,
            fps: 0.0,
            avg_fps: 0.0,
            frame_count: 0,
            elapsed: Duration::ZERO,
        }
    }

    pub fn update(&mut self, timestamp: Timestamp) {
        let now = timestamp;
        let delta = now - self.last_time;
        self.last_time = now;

        let seconds = delta.as_secs_f32();
        if seconds > 0.0 {
            self.fps = 1.0 / seconds;
        }

        self.frame_count += 1;
        self.elapsed += delta;
        self.avg_fps = self.calculate_avg();
    }

    pub fn get_fps(&self) -> f32 {
        self.fps
    }

    fn calculate_avg(&self) -> f32 {
        if self.elapsed.as_secs_f64() <= 0.0 {
            return 0.0;
        }
        self.frame_count as f32 / self.elapsed.as_secs_f32()
    }
}
