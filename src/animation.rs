use quicksilver::{graphics::Graphics, Result, Timer};

pub struct AnimationTimer {
    timer: Timer,
    at_frame: usize,
    frames_count: usize,
}

impl AnimationTimer {
    pub fn new(frames_count: usize, timer: Timer) -> AnimationTimer {
        Self {
            timer,
            at_frame: 0,
            frames_count,
        }
    }

    pub fn percent(&mut self) -> f32 {
        if self.at_frame >= self.frames_count {
            1.0
        } else {
            let frames_passed = self.timer.exhaust().map(usize::from).unwrap_or(0);

            match frames_passed.checked_add(self.at_frame) {
                Some(x) => {
                    self.at_frame = x;
                }
                None => self.at_frame = self.frames_count,
            }
            if self.at_frame > self.frames_count {
                self.at_frame = self.frames_count
            }

            self.at_frame as f32 / self.frames_count as f32
        }
    }

    pub fn is_ended(&self) -> bool {
        self.at_frame >= self.frames_count
    }
}

pub struct LinearConfig<T> {
    pub begin_state: T,
    pub timing: Timer,
    pub draw: Box<dyn Fn(&mut T, f32, &mut Graphics) -> Result<()>>,
    pub frame_count: usize,
}

impl<T> LinearConfig<T> {
    pub fn start(self) -> Linear<T> {
        Linear::new(self)
    }
}

pub struct Linear<T> {
    state: T,
    draw: Box<dyn Fn(&mut T, f32, &mut Graphics) -> Result<()>>,
    timer: AnimationTimer,
}

impl<T> Linear<T> {
    pub fn new(config: LinearConfig<T>) -> Self {
        Self {
            draw: config.draw,
            state: config.begin_state,
            timer: AnimationTimer::new(config.frame_count, config.timing),
        }
    }

    pub fn draw(&mut self, gfx: &mut Graphics) -> Result<()> {
        let percent = self.timer.percent();

        (self.draw)(&mut self.state, percent, gfx)
    }

    pub fn is_ended(&self) -> bool {
        return self.timer.is_ended();
    }
}
