use std::time::Instant;

use crate::text_renderer::{PixelPoint, TextInfo};

pub struct FpsCounter {
    last_frame: Instant,
    frame_times: Vec<f32>,
    max_samples: usize,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            frame_times: Vec::with_capacity(60),
            max_samples: 60,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.frame_times.push(delta);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
    }

    fn fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg_time: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        if avg_time > 0.0 { 1.0 / avg_time } else { 0.0 }
    }

    pub fn frame_time_ms(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg_time: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        avg_time * 1000.0
    }
}

impl TextInfo for FpsCounter {
    fn text(&self) -> String {
        format!("FPS:{:.1}\nFrame:{:.2}ms", self.fps(), self.frame_time_ms())
    }

    fn place(&self) -> PixelPoint {
        PixelPoint {
            x: 1000.0,
            y: 1000.0,
        }
    }
}
