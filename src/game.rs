use arrayvec::ArrayVec;
use vb_graphics::{Image, text::TextRenderer};
use vb_rt::sys::vip;

use crate::{assets, state::GameState};

struct Puzzle {
    name: &'static [u8],
    width: usize,
    height: usize,
    cells: &'static [u8],
}

#[rustfmt::skip]
const PUZZLE_CELLS: [u8; 15 * 15] = [
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

const PUZZLE: Puzzle = Puzzle {
    name: b"Bomberman Block",
    width: 15,
    height: 15,
    cells: &PUZZLE_CELLS,
};

#[derive(Debug, Clone, Copy)]
enum PuzzleCell {
    Empty { crossed: bool },
    Full,
}

pub struct Game {
    puzzle: &'static Puzzle,
    cells: [PuzzleCell; 15 * 15],
    cursor: (usize, usize),
    cursor_behavior: Option<PuzzleCell>,
    cursor_delay: u8,
    held: [u32; 4],
    solved: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            puzzle: &PUZZLE,
            cells: [PuzzleCell::Empty { crossed: false }; 15 * 15],
            cursor: (0, 0),
            cursor_behavior: None,
            cursor_delay: 0,
            held: [0; 4],
            solved: false,
        }
    }

    pub fn draw(&self) {
        const STEREO: vip::ObjectStereo = vip::ObjectStereo::new().with_jlon(true).with_jron(true);

        let mut next_world = 31;
        let world = vip::WORLDS.index(next_world);
        next_world -= 1;
        world.header().write(
            vip::WorldHeader::new()
                .with_bgm(vip::WorldMode::Object)
                .with_lon(true)
                .with_ron(true),
        );

        let mut obj_index = 1023;
        vip::SPT3.write(obj_index);
        let puzzle_left = 384 - 8 - (self.puzzle.width * 8);
        let puzzle_top = 224 - 8 - (self.puzzle.height * 8);

        for row in 0..self.puzzle.height {
            for col in 0..self.puzzle.width {
                let cell = self.cells[row * self.puzzle.width + col];
                let image = match cell {
                    PuzzleCell::Empty { crossed: false } => assets::SQUARE_EMPTY,
                    PuzzleCell::Empty { crossed: true } => assets::SQUARE_CROSS,
                    PuzzleCell::Full => assets::SQUARE_FULL,
                };
                let dst = (
                    (puzzle_left + col * 8) as i16,
                    (puzzle_top + row * 8) as i16,
                );
                obj_index = image.render_to_objects(obj_index, dst, STEREO);
            }
            let dst = (384 - 8, (puzzle_top + row * 8) as i16);
            obj_index = assets::SQUARE_RIGHT.render_to_objects(obj_index, dst, STEREO);
        }
        for col in 0..self.puzzle.width {
            let dst = ((puzzle_left + col * 8) as i16, 224 - 8);
            obj_index = assets::SQUARE_BOTTOM.render_to_objects(obj_index, dst, STEREO);
        }
        obj_index =
            assets::SQUARE_BOTTOM_RIGHT.render_to_objects(obj_index, (384 - 8, 224 - 8), STEREO);

        if !self.solved {
            let cursor_x = (puzzle_left + self.cursor.0 * 8) as i16;
            let cursor_y = (puzzle_top + self.cursor.1 * 8) as i16;
            obj_index =
                assets::SQUARE_HOVER.render_to_objects(obj_index, (cursor_x, cursor_y), STEREO);
        }

        for row in 0..self.puzzle.height {
            let range_start = row * self.puzzle.width;
            let range_end = range_start + self.puzzle.width;
            let indexes = range_start..range_end;
            let counts = self.line_count(indexes);
            if !counts.is_empty() {
                let mut num_x = (puzzle_left - 16) as i16;
                let num_y = (puzzle_top + row * 8) as i16;
                for num in counts.into_iter().rev() {
                    let image = NUMBERS[num as usize - 1];
                    obj_index = image.render_to_objects(obj_index, (num_x, num_y), STEREO);
                    num_x -= 8;
                }
            }
        }

        for col in 0..self.puzzle.width {
            let range_start = col;
            let range_end = col + self.puzzle.width * self.puzzle.height;
            let indexes = (range_start..range_end).step_by(self.puzzle.width);
            let counts = self.line_count(indexes);
            if !counts.is_empty() {
                let num_x = (puzzle_left + col * 8) as i16;
                let mut num_y = (puzzle_top - 16) as i16;
                for num in counts.into_iter().rev() {
                    let image = NUMBERS[num as usize - 1];
                    obj_index = image.render_to_objects(obj_index, (num_x, num_y), STEREO);
                    num_y -= 8;
                }
            }
        }

        vip::SPT2.write(obj_index);
        if self.solved {
            let world = vip::WORLDS.index(next_world);
            next_world -= 1;

            let text_width = assets::VIRTUAL_BOY.measure(self.puzzle.name) as i16;
            let text_height = assets::VIRTUAL_BOY.line_height as i16;

            world.header().write(
                vip::WorldHeader::new()
                    .with_bgm(vip::WorldMode::Normal)
                    .with_lon(true)
                    .with_ron(true)
                    .with_bg_map_base(1),
            );
            world.gx().write(96 - text_width / 2);
            world.gy().write(112 - text_height / 2);
            world.mx().write(0);
            world.my().write(0);
            world.w().write(text_width);
            world.h().write(text_height);
        }

        let world = vip::WORLDS.index(next_world);
        world.header().write(vip::WorldHeader::new().with_end(true));
    }

    fn line_count(&self, indexes: impl Iterator<Item = usize>) -> ArrayVec<u8, 16> {
        let mut result = ArrayVec::new();
        let mut last_count = 0;
        for index in indexes {
            let cell = self.puzzle.cells[index];
            if cell == 0 {
                if last_count != 0 {
                    result.push(last_count);
                }
                last_count = 0;
            } else {
                last_count += 1;
            }
        }
        if last_count != 0 {
            result.push(last_count);
        }

        result
    }

    pub fn update(&mut self, state: &GameState) {
        if self.solved {
            return;
        }

        let held = state.buttons_held();
        let mut cursor_moved = false;
        let mut handle_move = |button: bool, counter: &mut u32, delta: (isize, isize)| {
            if button {
                *counter += 1;
            } else {
                *counter = 0;
            }
            if *counter == 1 || *counter > 10 && self.cursor_delay == 0 {
                let new_x = (self.cursor.0 as isize + delta.0)
                    .rem_euclid(self.puzzle.width as isize) as usize;
                let new_y = (self.cursor.1 as isize + delta.1)
                    .rem_euclid(self.puzzle.height as isize) as usize;
                if self.cursor.0 != new_x || self.cursor.1 != new_y {
                    cursor_moved = true;
                }
                self.cursor = (new_x, new_y);
            }
        };

        handle_move(held.ll(), &mut self.held[0], (-1, 0));
        handle_move(held.lr(), &mut self.held[1], (1, 0));
        handle_move(held.lu(), &mut self.held[2], (0, -1));
        handle_move(held.ld(), &mut self.held[3], (0, 1));

        if cursor_moved {
            self.cursor_delay = 4;
        } else {
            self.cursor_delay = self.cursor_delay.saturating_sub(1);
        }

        let pressed = state.buttons_pressed();
        let index = self.cursor.1 * self.puzzle.width + self.cursor.0;
        if pressed.b() {
            let new_cell = match self.cells[index] {
                PuzzleCell::Empty { crossed: true } => PuzzleCell::Empty { crossed: false },
                _ => PuzzleCell::Empty { crossed: true },
            };
            self.cursor_behavior = Some(new_cell);
        }
        if pressed.a() {
            let new_cell = match self.cells[index] {
                PuzzleCell::Full => PuzzleCell::Empty { crossed: false },
                _ => PuzzleCell::Full,
            };
            self.cursor_behavior = Some(new_cell);
        }
        if !held.a() && !held.b() {
            self.cursor_behavior = None;
        }
        if let Some(behavior) = self.cursor_behavior {
            self.cells[index] = behavior;
            if self.has_been_solved() {
                let mut renderer = TextRenderer::new(&assets::VIRTUAL_BOY, 128, (24, 14));
                renderer.draw_text(self.puzzle.name);
                renderer.render_to_bgmap(1, (0, 0));
                self.solved = true;
            }
        }
    }

    fn has_been_solved(&self) -> bool {
        self.puzzle
            .cells
            .iter()
            .zip(self.cells)
            .all(|(solution, cell)| {
                let expected = *solution;
                let actual = match cell {
                    PuzzleCell::Empty { .. } => 0,
                    PuzzleCell::Full => 1,
                };
                expected == actual
            })
    }
}

const NUMBERS: [&Image; 20] = [
    &assets::NUMBER_1,
    &assets::NUMBER_2,
    &assets::NUMBER_3,
    &assets::NUMBER_4,
    &assets::NUMBER_5,
    &assets::NUMBER_6,
    &assets::NUMBER_7,
    &assets::NUMBER_8,
    &assets::NUMBER_9,
    &assets::NUMBER_10,
    &assets::NUMBER_11,
    &assets::NUMBER_12,
    &assets::NUMBER_13,
    &assets::NUMBER_14,
    &assets::NUMBER_15,
    &assets::NUMBER_16,
    &assets::NUMBER_17,
    &assets::NUMBER_18,
    &assets::NUMBER_19,
    &assets::NUMBER_20,
];

#[expect(unused)]
const NUMBERS_DIM: [&Image; 20] = [
    &assets::NUMBER_1_DIM,
    &assets::NUMBER_2_DIM,
    &assets::NUMBER_3_DIM,
    &assets::NUMBER_4_DIM,
    &assets::NUMBER_5_DIM,
    &assets::NUMBER_6_DIM,
    &assets::NUMBER_7_DIM,
    &assets::NUMBER_8_DIM,
    &assets::NUMBER_9_DIM,
    &assets::NUMBER_10_DIM,
    &assets::NUMBER_11_DIM,
    &assets::NUMBER_12_DIM,
    &assets::NUMBER_13_DIM,
    &assets::NUMBER_14_DIM,
    &assets::NUMBER_15_DIM,
    &assets::NUMBER_16_DIM,
    &assets::NUMBER_17_DIM,
    &assets::NUMBER_18_DIM,
    &assets::NUMBER_19_DIM,
    &assets::NUMBER_20_DIM,
];
