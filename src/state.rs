use vb_rt::sys::hardware;

pub struct GameState {
    curr_pressed: hardware::GamePadData,
    prev_pressed: hardware::GamePadData,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            curr_pressed: hardware::GamePadData::new(),
            prev_pressed: hardware::GamePadData::new(),
        }
    }

    pub fn buttons_held(&self) -> hardware::GamePadData {
        self.curr_pressed
    }

    pub fn buttons_pressed(&self) -> hardware::GamePadData {
        hardware::GamePadData::from_bits(
            self.curr_pressed.into_bits() & !self.prev_pressed.into_bits(),
        )
    }

    pub fn update(&mut self) {
        let pressed = hardware::read_controller();
        self.prev_pressed = self.curr_pressed;
        self.curr_pressed = pressed;
    }
}
