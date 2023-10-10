use std::time::Instant;

pub struct Timer {
    start_time: Instant,
    name: String    
}

impl Timer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            start_time: Instant::now(),
            name: name.into()
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let end_time = Instant::now();
        let elapsed = end_time.duration_since(self.start_time);
        log::info!("{}: {}ns", self.name, elapsed.as_nanos());
    }
}

