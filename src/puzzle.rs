pub struct Puzzle {
    pub name: &'static [u8],
    pub width: usize,
    pub height: usize,
    pub cells: &'static [u8],
}

pub const EMPTY: Puzzle = Puzzle {
    name: &[],
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
    width: 15,
    height: 15,
    cells: &BOMBERMAN_BLOCK_CELLS,
};

pub const PUZZLES: [Puzzle; 15] = [
    GOLF_BALL,
    BOMBERMAN_BLOCK,
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
    EMPTY,
];
