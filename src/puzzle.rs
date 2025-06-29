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

const fn format_puzzle<const N: usize>(input: &[u8]) -> [u8; N] {
    let mut result = [0; N];
    let mut src_index = 0;
    let mut dst_index = 0;
    while src_index < input.len() {
        if input[src_index] == b'x' {
            result[dst_index] = 1;
            dst_index += 1;
        } else if input[src_index] == b'-' {
            dst_index += 1;
        }
        src_index += 1;
    }
    assert!(dst_index == N);
    result
}

macro_rules! puzzle {
    ($name:expr, $source:expr, ($width:expr, $height:expr), $puzzle:expr) => {{
        const PUZZLE_CELLS: [u8; $width * $height] = format_puzzle($puzzle);
        Puzzle {
            name: $name,
            source: $source,
            width: $width,
            height: $height,
            cells: &PUZZLE_CELLS,
        }
    }};
}

pub const GOLF_BALL: Puzzle = puzzle!(
    b"Golf Ball",
    b"Golf",
    (5, 5),
    b"
    --x--
    -xxx-
    xxxxx
    -xxx-
    --x--"
);

pub const BOMBERMAN_BLOCK: Puzzle = puzzle!(
    b"Bomberman Block",
    b"Panic Bomber",
    (15, 15),
    b"
    ------------xx-
    -----xxxxx--xx-
    ---xxxxxxxxx---
    --xx------xxx--
    --x--x--x--xx--
    -x---x--x---xx-
    -x---x--x----x-
    xx-----------xx
    xx-----------xx
    -x---xxxxx---x-
    -xx-xxxxxxx-xx-
    --xxxxxxxxxxx--
    --xxxxxxxxxxx--
    ---xxxxxxxxx---
    ---------------"
);

pub const CHALVO: Puzzle = puzzle!(
    b"Chalvo",
    b"Bound High",
    (15, 15),
    b"
    -----xxxxx-----
    ---xxx---xxx---
    --x---------x--
    -xx---------xx-
    -xx---------xx-
    x-xxxx---xxxx-x
    x-xxx-x-xxx-x-x
    x-xxx-x-xxx-x-x
    x-x---x-x---x-x
    x-xxxx---xxxx-x
    -xx---------xx-
    -xxxx-----xxxx-
    --x--xxxxx--x--
    --xxx-----xxx--
    ----xxxxxxx----
    "
);

pub const HOMING_MISSILES: Puzzle = puzzle!(
    b"Homing Missiles",
    b"Red Alarm",
    (20, 20),
    b"
    ---------x----------
    ---------x----------
    -x------xxx------x--
    -x------xxx------x--
    xxx-----x-x-----xxx-
    xxx-------------xxx-
    x-x------x------x-x-
    ---------x----------
    -x-------x-------x--
    -x----x--x--x----x--
    ------x-xxx-x-------
    ------x-xxx-x-------
    -x----xxxxxxx----x--
    -x---xxxxxxxxx---x--
    -----xxxxxxxxx------
    --x-xxxxxxxxxxx-x---
    ---xxxxxxxxxxxxx----
    ----xxx-xxx-xxx-----
    -----xx--x--xx------
    ------x-----x-------"
);

pub const PUZZLES: [Puzzle; 16] = [
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
    CHALVO,
];

pub const ICON_CHARS: [vip::Character; PUZZLES.len() * 25] =
    unsafe { core::mem::transmute(menu_icon_chars(PUZZLES)) };
pub const ICON_CHAR_OFFSET: usize = 1024;
pub const ICON_CELLS: [[vip::Cell; 25]; PUZZLES.len()] = menu_icon_cells(1024);
pub const ICONS: [Image; PUZZLES.len()] = menu_icon_images(&ICON_CELLS);
