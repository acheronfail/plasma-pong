use std::time::Instant;

pub struct FpsCounter {
    last_check: Instant,
    frames_since_last_check: f32,
    last_fps: f32,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            last_check: Instant::now(),
            frames_since_last_check: 1.0,
            last_fps: f32::INFINITY,
        }
    }

    pub fn update(&mut self) {
        let time = self.last_check.elapsed().as_secs_f32();
        if time < 0.5 {
            self.frames_since_last_check += 1.0;
        } else {
            self.last_fps = (1. / time) * self.frames_since_last_check;
            self.frames_since_last_check = 0.0;
            self.last_check = Instant::now();
        }
    }

    pub fn fps(&self) -> f32 {
        self.last_fps
    }
}
