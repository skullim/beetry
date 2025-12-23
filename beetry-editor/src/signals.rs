pub struct RequestRender {
    // dummy flag to trigger value change
    flag: bool,
}

impl RequestRender {
    pub fn new() -> Self {
        Self { flag: false }
    }

    pub fn request(&mut self) {
        self.flag = !self.flag;
    }
}
