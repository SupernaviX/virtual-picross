use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoroshiro128PlusPlus;
use vb_rt::sys::hardware;

const DPAD_OFFSETS: [u16; 4] = [8, 9, 10, 11];

pub struct GameState {
    curr_pressed: hardware::GamePadData,
    prev_pressed: hardware::GamePadData,
    curr_held: [u32; DPAD_OFFSETS.len()],
    rand: Xoroshiro128PlusPlus,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            curr_pressed: hardware::GamePadData::new(),
            prev_pressed: hardware::GamePadData::new(),
            curr_held: [0; DPAD_OFFSETS.len()],
            rand: Xoroshiro128PlusPlus::seed_from_u64(0),
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

    pub fn directions_held(&self) -> hardware::GamePadData {
        let mut bits = 0;
        for (counter, button_offset) in self.curr_held.iter().zip(DPAD_OFFSETS) {
            if *counter == 1 || *counter >= 10 {
                bits |= 1 << button_offset;
            }
        }

        hardware::GamePadData::from_bits(bits)
    }

    pub fn update(&mut self) {
        let pressed = hardware::read_controller();
        self.prev_pressed = self.curr_pressed;
        self.curr_pressed = pressed;
        if self.prev_pressed.into_bits() != self.curr_pressed.into_bits() {
            let _: u32 = self.rand.random();
        }

        let raw_bits = pressed.into_bits();
        for (counter, button_offset) in self.curr_held.iter_mut().zip(DPAD_OFFSETS) {
            let down = (raw_bits >> button_offset) & 1 == 1;
            if down {
                *counter += 1;
            } else {
                *counter = 0;
            }
        }
    }

    pub fn rand(&mut self) -> &mut impl Rng {
        &mut self.rand
    }
}
