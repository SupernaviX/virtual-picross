use vb_rt::sys::sram;

use crate::puzzle::PUZZLES;

struct Fletcher {
    s1: u16,
    s2: u16,
}
impl Fletcher {
    fn new() -> Self { Self { s1: 0, s2: 0 }}
    fn add(&mut self, byte: u8) {
        self.s1 = (self.s1 + byte as u16) % 255;
        self.s2 = (self.s2 + self.s1) % 255;
    }
    fn add_many(&mut self, bytes: impl IntoIterator<Item = u8>) {
        for byte in bytes {
            self.add(byte);
        }
    }
    fn finish(self) -> u16 {
        self.s2 << 8 | self.s1
    }
}

pub struct SaveData {
    pub times: [Option<u32>; PUZZLES.len()],
}

impl SaveData {
    pub fn load() -> Self {
        let mut fletcher = Fletcher::new();
        let times = core::array::from_fn(|index| {
            let mut bytes = [0; 4];
            sram::SRAM.read_slice(&mut bytes, 256 + index * 4);
            fletcher.add_many(bytes);
            let time = u32::from_le_bytes(bytes);
            if time > 0 { Some(time) } else { None }
        });

        let expected_checksum = fletcher.finish();
        let actual_checksum = u16::from_le_bytes(sram::SRAM.read_array(0));
        if expected_checksum == actual_checksum {
            Self { times }
        } else {
            for index in 0..PUZZLES.len() * 4 {
                sram::SRAM.index(256 + index).write(0);
            }
            Self { times: [None; PUZZLES.len()] }
        }
    }

    pub fn save_time(&mut self, index: usize, time: u32) {
        self.times[index] = Some(time);
        let mut fletcher = Fletcher::new();
        for time in self.times {
            fletcher.add_many(time.unwrap_or_default().to_le_bytes());
        }
        let checksum = fletcher.finish();
        sram::SRAM.write_slice(&time.to_le_bytes(), 256 + index * 4);
        sram::SRAM.write_slice(&checksum.to_le_bytes(), 0);
    }
}
