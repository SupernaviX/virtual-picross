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

pub const TETRIS_BLOCK: Puzzle = puzzle!(
    b"Tetris Block",
    b"V-Tetris",
    (5, 5),
    b"
    xxxxx
    x---x
    x-x-x
    x---x
    xxxxx"
);

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

pub const HEART: Puzzle = puzzle!(
    b"Heart",
    b"Virtual Boy Wario Land",
    (5, 5),
    b"
    -x-x-
    xxxxx
    xxxxx
    -xxx-
    --x--"
);

pub const BOWLING_PIN: Puzzle = puzzle!(
    b"Bowling Pin",
    b"Nester's Funky Bowling",
    (5, 10),
    b"
    --x--
    -x-x-
    -x-x-
    -xxx-
    -x-x-
    xx-xx
    x-x-x
    x---x
    -x-x-
    -xxx-"
);

pub const VB: Puzzle = puzzle!(
    b"VB",
    b"Virtual Boy",
    (10, 5),
    b"
    x--x-xxx--
    x--x-x--x-
    x--x-xxx--
    xxxx-x--x-
    -xx--xxx--"
);

pub const MYUU: Puzzle = puzzle!(
    b"Myuu",
    b"Virtual Lab",
    (10, 10),
    b"
    ----------
    ----------
    ----------
    -xxx--xxx-
    xxxxxxxxxx
    xxxxxxxxxx
    -xxxxxxxx-
    ---xxxx---
    ----xx----
    ----xx----"
);

pub const TETRIS_SQUARE: Puzzle = puzzle!(
    b"Tetris Square",
    b"3D Tetris",
    (10, 10),
    b"
    ---xxxxxxx
    --xx----xx
    -x-x---x-x
    xxxxxxx--x
    x--x--x--x
    x--x--x--x
    x--xxxxxxx
    x-x---x-x-
    xx----xx--
    xxxxxxx---"
);

pub const BOMBERMAN: Puzzle = puzzle!(
    b"Bomberman",
    b"Panic Bomber",
    (10, 10),
    b"
    xx--------
    xxxxxxxx--
    -x------x-
    x--xxxxx-x
    x-x-x-x-xx
    x-x-x-x-xx
    x-x-----xx
    x--xxxxx-x
    -x------x-
    --xxxxxx--"
);

pub const UFO: Puzzle = puzzle!(
    b"UFO",
    b"Galactic Pinball",
    (10, 10),
    b"
    ----------
    ----xx----
    ---x-xx---
    --x-xx-x--
    -xxxx-xxx-
    xxxxxxxxxx
    x--xxx---x
    -xx----xx-
    ---xxxx---
    ----------"
);

pub const ORB: Puzzle = puzzle!(
    b"Orb",
    b"Innsmouth no Yakata",
    (10, 10),
    b"
    ---xxxx---
    --xxxxxx--
    -xx-xxxxx-
    -x---xxxx-
    -xx-xxxxx-
    -xxxxxxxx-
    --xxxxxx--
    ---xxxx---
    ----------
    ---xxxx---"
);

pub const KOOPA: Puzzle = puzzle!(
    b"Koopa",
    b"Mario Clash",
    (10, 10),
    b"
    -------xx-
    ------x--x
    ---xx-x-xx
    --xxxx---x
    -xxxxxx--x
    -xxxxxxxx-
    -xxxxxx-xx
    x------x--
    -xxxxxx---
    xx----xx--"
);

pub const ATOLLER: Puzzle = puzzle!(
    b"Atoller",
    b"Waterworld",
    (15, 10),
    b"
    ---------------
    ---------------
    ---------------
    ------xx-----x-
    xx----xx----xx-
    -xxxxx--xxxxx--
    ---xxxxxxxx----
    ----xxxxxx-----
    ---xxxxxxxx----
    ---------------"
);

pub const ZAKU_II: Puzzle = puzzle!(
    b"MS-06 Zaku II",
    b"SD Gundam Dimension War",
    (15, 10),
    b"
    -----xxxxx-----
    ---xxxxxxxxx---
    --xxxxxxxxxxx--
    --xxxxxxxxxxxx-
    -xx-xxxxxxxx-x-
    -xx-x----xx--xx
    xx--x---xxxx--x
    xxxxx----xx--xx
    xxxxxxxxx--xxxx
    --xxxxxxxxxxxx-"
);

pub const MEDIUM_INVADER: Puzzle = puzzle!(
    b"Medium Invader",
    b"Space Invaders Virtual Collection",
    (15, 10),
    b"
    ---------------
    ----x-----x----
    -----x---x-----
    ----xxxxxxx----
    ---xx-xxx-xx---
    --xxxxxxxxxxx--
    --x-xxxxxxx-x--
    --x-x-----x-x--
    -----xx-xx-----
    ---------------"
);

pub const PITCHER: Puzzle = puzzle!(
    b"Pitcher",
    b"Virtual League Baseball",
    (10, 15),
    b"
    ----------
    ----------
    ---xx-----
    ---xx--x--
    --x--xx---
    --xxxx----
    -x-xxx----
    -x-xxx----
    ---xx-----
    --xxxx----
    --xx-xx---
    --xx-xx---
    --x---x---
    --x---x---
    ----------"
);

pub const LURE: Puzzle = puzzle!(
    b"Lure",
    b"Virtual Fishing",
    (15, 10),
    b"
    ------xxxx-----
    --xxxx----xxxxx
    -xx-----xxxxxx-
    x--x---xxxx---x
    xx--xxxxxx--x-x
    xxxxxxxxx-x-xx-
    -xxx--x---x--xx
    ----xx--x-x-x--
    ---xx----xxx---
    --xx-----------"
);

pub const BOWLER: Puzzle = puzzle!(
    b"Bowler",
    b"Virtual Bowling",
    (10, 15),
    b"
    x---------
    x--xx-----
    x-xxxx----
    x--xx-----
    xxxxxxxx--
    -xxxx---xx
    --xxxx----
    ---xxxx---
    ---xxxx---
    --xxxxx---
    --xx--xx--
    ---x---xx-
    ---x----xx
    -xxx---xxx
    xxxx------"
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

pub const JACK_SKELTON: Puzzle = puzzle!(
    b"Jack Skelton",
    b"Jack Bros.",
    (15, 15),
    b"
    ----xxx-xxx----
    ----xxxxxxx----
    ---xxxx--xxx---
    ---xxxxx--xx---
    -xxxxxxx--xxxx-
    x--xxxxxxxxx--x
    x---xxxxxx----x
    -xxx-------xxx-
    --x-xxxxxxx-x--
    --x--xx-xx--x--
    --x----x----x--
    --x-x-x-x-x-x--
    ---x-xxxxx-x---
    ----x--x--x----
    -----xx-xx-----"
);

pub const CAT: Puzzle = puzzle!(
    b"Cat",
    b"Virtual Boy Wario Land",
    (15, 15),
    b"
    ----xx---xx----
    ---x--xxx--x---
    ---x-x-x-x-x---
    --x-xxx-xxx-x--
    --x--xx-xx--xx-
    --x----x----x-x
    -xxx-xx-xx-xx-x
    x--xx----xxx--x
    x---xxxxxx----x
    -xx--xxxx---xx-
    --xx--xxx-----x
    -x-xxxxxxx----x
    -x--xx--xxxx--x
    --x-xxx--xxx-x-
    --xx-xxxxx-xxx-"
);

pub const REPAIR_DRONE: Puzzle = puzzle!(
    b"Repair Drone",
    b"Vertical Force",
    (15, 15),
    b"
    ------xxx------
    -----xxxxx-----
    ----xx-x-xx----
    ---xx-x-x-xx---
    ---x-x---x-x---
    --x-xx-x-xx-x--
    --x-x-xxx-x-x--
    --x-x-xxx-x-x--
    --x-x--x--x-x--
    ---x-xx-xx-x---
    ----x--x--x----
    -x--x-x-x-x--x-
    xxx-xx-x-xx-xxx
    xxxxxxxxxxxxxxx
    x-x--x---x--x-x"
);

pub const YOSHI: Puzzle = puzzle!(
    b"Yoshi",
    b"Mario's Tennis",
    (15, 15),
    b"
    xxx-xxx-xx-xxxx
    x--x---x--x---x
    x--x-xx-xxx---x
    x--x-xx-xxx---x
    x-x-x-xxxxxx--x
    x-x--x------x-x
    -x--x----x-x-x-
    x-------------x
    x--xx---------x
    x---x---------x
    -xx--x-------x-
    x-xx--xxxxxxx-x
    x--xx-----xx--x
    x---xxxxxxx---x
    xxxx-------xxxx"
);

pub const P_TRON: Puzzle = puzzle!(
    b"P-Tron",
    b"Space Squash",
    (15, 15),
    b"
    xxxxxxx-xxxxxxx
    xxxxxx---xxxxxx
    x--xx--x--xx--x
    xx---xxxxx---xx
    xxx-x-xxx-x-xxx
    xx-x-x-x-x-x-xx
    x--x--xxx--x--x
    x-x-xxxxxxx-x-x
    -xx---xxx---xx-
    -xxx-x---x-xxx-
    --xx--xxx--xx--
    ---xx--x--xx---
    ---x--x-x--x---
    xxx---xxx---xxx
    xx---xxxxx---xx"
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
    ----xxxxxxx----"
);

pub const MARIO: Puzzle = puzzle!(
    b"Mario",
    b"Mario Clash",
    (15, 15),
    b"
    ----xxxxxxxx---
    ----x-x--xxxx--
    --xx--x----xxx-
    -xx-xx------xxx
    x-------------x
    xx----x-------x
    -xxxxx-x------x
    ---x-x-xxxx--x-
    -xx--x-x--xxxx-
    x-----x--xxxxxx
    x--xxx----xxxxx
    -xxxxxx---x--xx
    --xxxx------xxx
    ---x----xxxx---
    ----xxxxxxxx---"
);

pub const PAGERO: Puzzle = puzzle!(
    b"Pagero",
    b"Teleroboxer",
    (15, 15),
    b"
    xxxx---x---xxxx
    xxxxxxx-xxxxxxx
    -xxxxx---xxxxx-
    -xxxxx---xxxxx-
    --xxxxx-xxxxx--
    --xx--xxx--xx--
    --x-xx-x-xx-x--
    x-xx--x-x--xx-x
    xxx-xx---xx-xxx
    x-x----x----x-x
    --xxx-xxx-xxx--
    --xx-x---x-xx--
    ---x-xxxxx-x---
    ----xx---xx----
    ------xxx------"
);

pub const WARIO: Puzzle = puzzle!(
    b"Wario",
    b"Virtual Boy Wario Land",
    (15, 15),
    b"
    --xx-xxxxxx----
    -xxxxxxxxxxxx--
    xxx-xxxxx--xxx-
    -xx---------xxx
    -xxxxx--xxx--xx
    x-xxxxxxxx-xxx-
    x-x-x-x-x--xx--
    x-x--x-x--xx-x-
    x--xx---xxx--x-
    -x--xx-xxxxx--x
    x-xxxxxx-x--x-x
    x-x-x-x--x--x-x
    -x-xx-x---x-xx-
    --x-xxxxxx-xxx-
    ---x------xxx--"
);

pub const MASK_GUY: Puzzle = puzzle!(
    b"Mask Guy",
    b"Virtual Boy Wario Land",
    (20, 15),
    b"
    ----xxxxxxxx--------
    --xxx-x----xx-------
    -xxxx--x--x-x-------
    xxxxx---xx--x-------
    xxxxx--x--x-x-------
    xxxxxx-x--x-x-xxx---
    xx--x-x-xx--x-xxxxx-
    x--xx--xxxxxxxxxxxxx
    x---x--xxxxxxxxxxxx-
    x--xx---xx--x-xxx---
    xxxxx-x-x-x-x-------
    -xxxxx-x-x-xx-------
    x--xx-x-x-x-x-------
    -xx-xx-xxx-x--------
    ---xx----xxx--------"
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

pub const MEGU: Puzzle = puzzle!(
    b"Megu-tan",
    b"Virtual Lab",
    (20, 20),
    b"
    ----x---xxxxxx------
    ---x-xxx------xx----
    ---x----xxxx----x---
    ----x--x----xx--x---
    ---x--x-x--x--x-x---
    ---x--x-xx-xx-x-x---
    ----xxx------x-x----
    ----xx-x-x--x-xx----
    ---xx-xxx--xxx-xx---
    --xx-x---xx---x-xx--
    -xx-xxx------xxx-xx-
    -x--x-xx-xx-xx-x--x-
    -x-x-x-xxxxxx-x-x-x-
    -x-x-x-x-xx-x-x-x-x-
    -x--x-x--xx--x-x--x-
    -x----x-xxxx-x----x-
    -x-x-x-xxxxxxx-x-x--
    --x-x-xxx--xxxx-x---
    ------xx----xx------
    -------xx--xx-------"
);

pub const PUZZLES: [Puzzle; 30] = [
    TETRIS_BLOCK,
    GOLF_BALL,
    HEART,
    BOWLING_PIN,
    VB,
    MYUU,
    TETRIS_SQUARE,
    BOMBERMAN,
    UFO,
    ORB,
    KOOPA,
    ATOLLER,
    ZAKU_II,
    MEDIUM_INVADER,
    PITCHER,
    LURE,
    BOWLER,
    BOMBERMAN_BLOCK,
    JACK_SKELTON,
    CAT,
    REPAIR_DRONE,
    YOSHI,
    P_TRON,
    CHALVO,
    MARIO,
    PAGERO,
    WARIO,
    MASK_GUY,
    HOMING_MISSILES,
    MEGU,
];

pub const ICON_CHARS: [vip::Character; PUZZLES.len() * 25] =
    unsafe { core::mem::transmute(menu_icon_chars(PUZZLES)) };
pub const ICON_CHAR_OFFSET: usize = 1024;
pub const ICON_CELLS: [[vip::Cell; 25]; PUZZLES.len()] = menu_icon_cells(1024);
pub const ICONS: [Image; PUZZLES.len()] = menu_icon_images(&ICON_CELLS);
