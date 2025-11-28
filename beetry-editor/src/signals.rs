pub struct RequestRerender {
    // dummy flag to trigger value change
    flag: bool,
}

impl RequestRerender {
    pub fn new() -> Self {
        Self { flag: false }
    }

    pub fn request(&mut self) {
        self.flag = !self.flag;
    }
}
