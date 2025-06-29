use vb_graphics::Image;
use vb_rt::sys::vip;

pub struct Puzzle {
    pub name: &'static [u8],
    pub source: &'static [u8],
    pub width: usize,
    pub height: usize,
    pub cells: &'static [u8],
}

const fn menu_icon_row(mut cells: &[u8]) -> [u16; 5] {
    let offset = 20 - cells.len();
    let mut result = [0; 5];
    let mut dst_index = offset;
    while let Some((&cell, rest)) = cells.split_first() {
        cells = rest;
        let target_dst_index = dst_index + 2;
        while dst_index < target_dst_index {
            if cell != 0 {
                let dst_hw = &mut result[dst_index / 8];
                let dst_offset = dst_index % 8;
                *dst_hw |= 0b10 << (dst_offset * 2);
            }
            dst_index += 1;
        }
    }
    result
}

const fn menu_icon(mut cells: &[u8], width: usize, height: usize) -> [vip::Character; 25] {
    let offset = 20 - height;
    let mut result = [vip::Character([0; 8]); 25];
    if width == 0 || height == 0 {
        return result;
    }
    let mut dst_index = offset;
    while let Some((row, rest)) = cells.split_at_checked(width) {
        let src_row = menu_icon_row(row);
        cells = rest;

        let target_dst_index = dst_index + 2;
        while dst_index < target_dst_index {
            let mut col = 0;
            let target_col = 5;
            while col < target_col {
                let char = &mut result[5 * (dst_index / 8) + col];
                let row = src_row[col];
                char.0[dst_index % 8] = row;
                col += 1;
            }
            dst_index += 1;
        }
    }
    result
}

const fn menu_icon_chars<const N: usize>(puzzles: [Puzzle; N]) -> [[vip::Character; 25]; N] {
    let mut result = [[vip::Character([0; 8]); 25]; N];
    let mut index = 0;
    while index < N {
        let puzzle = &puzzles[index];
        result[index] = menu_icon(puzzle.cells, puzzle.width, puzzle.height);
        index += 1;
    }
    result
}

const fn menu_icon_cells<const N: usize>(offset: u16) -> [[vip::Cell; 25]; N] {
    let mut result = [[vip::Cell::new(); 25]; N];
    let mut index = 0;
    let mut char_index = offset;
    while index < N {
        let cell = &mut result[index];
        index += 1;

        let mut index = 0;
        while index < 25 {
            cell[index] = vip::Cell::new().with_character(char_index);
            char_index += 1;
            index += 1;
        }
    }
    result
}

const fn menu_icon_images<const N: usize>(cells: &'static [[vip::Cell; 25]; N]) -> [Image; N] {
    let mut result = [const {
        Image {
            width_cells: 5,
            height_cells: 5,
            data: &[],
        }
    }; N];
    let mut index = 0;
    while index < N {
        result[index].data = &cells[index];
        index += 1;
    }
    result
}

pub const EMPTY: Puzzle = Puzzle {
    name: &[],
    source: &[],
    width: 0,
    height: 0,
    cells: &[],
};

#[rustfmt::skip]
const GOLF_BALL_CELLS: [u8; 5 * 5] = [
    0, 0, 1, 0, 0,
    0, 1, 1, 1, 0,
    1, 1, 1, 1, 1,
    0, 1, 1, 1, 0,
    0, 0, 1, 0, 0,
];

pub const GOLF_BALL: Puzzle = Puzzle {
    name: b"Golf Ball",
    source: b"Golf",
    width: 5,
    height: 5,
    cells: &GOLF_BALL_CELLS,
};

#[rustfmt::skip]
const BOMBERMAN_BLOCK_CELLS: [u8; 15 * 15] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0,
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0,
    0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0,
    0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0,
    0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 1, 0,
    0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0,
    1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
    1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
    0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0,
    0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0,
    0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,
    0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const BOMBERMAN_BLOCK: Puzzle = Puzzle {
    name: b"Bomberman Block",
    source: b"Panic Bomber",
    width: 15,
    height: 15,
    cells: &BOMBERMAN_BLOCK_CELLS,
};

#[rustfmt::skip]
pub const HOMING_MISSILES_CELLS: [u8; 20 * 20] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0,
    0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0,
    1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0,
    1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0,
    1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
    0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0,
    0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0,
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
    0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
];

pub const HOMING_MISSILES: Puzzle = Puzzle {
    name: b"Homing Missiles",
    source: b"Red Alarm",
    width: 20,
    height: 20,
    cells: &HOMING_MISSILES_CELLS,
};

pub const PUZZLES: [Puzzle; 15] = [
    GOLF_BALL,
    BOMBERMAN_BLOCK,
    HOMING_MISSILES,
    EMPTY,
    EMPTY,
    EMPTY,
    EMPTY,
    EMPTY,
    EMPTY,
    EMPTY,
    EMPTY,
    EMPTY,
    EMPTY,
    EMPTY,
    EMPTY,
];

pub const ICON_CHARS: [vip::Character; PUZZLES.len() * 25] =
    unsafe { core::mem::transmute(menu_icon_chars(PUZZLES)) };
pub const ICON_CHAR_OFFSET: usize = 1024;
pub const ICON_CELLS: [[vip::Cell; 25]; PUZZLES.len()] = menu_icon_cells(1024);
pub const ICONS: [Image; PUZZLES.len()] = menu_icon_images(&ICON_CELLS);
