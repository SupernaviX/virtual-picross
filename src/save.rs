use vb_rt::sys::sram;

use crate::puzzle::PUZZLES;

pub fn load_times() -> [Option<u32>; PUZZLES.len()] {
    let mut bytes = [0; 4];
    core::array::from_fn(|index| {
        sram::SRAM.read_slice(&mut bytes, 256 + index * 4);
        let time = u32::from_le_bytes(bytes);
        if time > 0 { Some(time) } else { None }
    })
}

pub fn save_time(index: usize, time: u32) {
    let bytes = time.to_le_bytes();
    sram::SRAM.write_slice(&bytes, 256 + index * 4);
}
