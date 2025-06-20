use arrayvec::ArrayVec;
use vb_graphics::{Image, text::TextRenderer};
use vb_rt::sys::vip;

use crate::{
    assets,
    puzzle::{EMPTY, Puzzle},
    state::GameState,
};

#[derive(Debug, Clone, Copy)]
enum PuzzleCell {
    Empty,
    Cross,
    Full,
}

const MAX_PUZZLE_SIZE: usize = 15;

pub struct Game {
    puzzle: &'static Puzzle,
    cells: [PuzzleCell; MAX_PUZZLE_SIZE * MAX_PUZZLE_SIZE],
    row_numbers: ArrayVec<ArrayVec<(u8, bool), MAX_PUZZLE_SIZE>, MAX_PUZZLE_SIZE>,
    col_numbers: ArrayVec<ArrayVec<(u8, bool), MAX_PUZZLE_SIZE>, MAX_PUZZLE_SIZE>,
    cursor: (usize, usize),
    cursor_behavior: Option<PuzzleCell>,
    cursor_delay: u8,
    held: [u32; 4],
    solved: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            puzzle: &EMPTY,
            cells: [PuzzleCell::Empty; MAX_PUZZLE_SIZE * MAX_PUZZLE_SIZE],
            row_numbers: ArrayVec::new(),
            col_numbers: ArrayVec::new(),
            cursor: (0, 0),
            cursor_behavior: None,
            cursor_delay: 0,
            held: [0; 4],
            solved: false,
        }
    }

    pub fn load_puzzle(&mut self, puzzle: &'static Puzzle) {
        self.puzzle = puzzle;
        for cell in self.cells.iter_mut().take(puzzle.width * puzzle.height) {
            *cell = PuzzleCell::Empty;
        }
        self.row_numbers = (0..self.puzzle.height)
            .map(|row| self.row_count(row))
            .collect();
        self.col_numbers = (0..self.puzzle.width)
            .map(|col| self.col_count(col))
            .collect();
        self.cursor = (0, 0);
        self.cursor_behavior = None;
        self.cursor_delay = 0;
        self.held = [0; 4];
        self.solved = false;
    }

    pub fn size_pixels(&self) -> (usize, usize) {
        let width = (self.puzzle.width * 8)
            + 8
            + self
                .col_numbers
                .iter()
                .map(|n| n.len() * 8)
                .max()
                .unwrap_or_default();
        let height = (self.puzzle.height * 8)
            + 8
            + self
                .row_numbers
                .iter()
                .map(|n| n.len() * 8)
                .max()
                .unwrap_or_default();
        (width, height)
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

        let (width_pixels, height_pixels) = self.size_pixels();
        let x_offset = (384 - width_pixels) / 2;
        let y_offset = (224 - height_pixels) / 2;

        let puzzle_right = 384 - 8 - x_offset;
        let puzzle_bottom = 224 - 8 - y_offset;
        let puzzle_left = puzzle_right - (self.puzzle.width * 8);
        let puzzle_top = puzzle_bottom - (self.puzzle.height * 8);

        for row in 0..self.puzzle.height {
            for col in 0..self.puzzle.width {
                let col_bright = col > 0 && col % 5 == 0;
                let row_bright = row > 0 && row % 5 == 0;
                let cell = self.cells[row * self.puzzle.width + col];
                let image = match (col_bright, row_bright, cell) {
                    (false, false, PuzzleCell::Empty) => assets::SQUARE_DD_EMPTY,
                    (false, false, PuzzleCell::Cross) => assets::SQUARE_DD_CROSS,
                    (false, false, PuzzleCell::Full) => assets::SQUARE_DD_FULL,
                    (false, true, PuzzleCell::Empty) => assets::SQUARE_DB_EMPTY,
                    (false, true, PuzzleCell::Cross) => assets::SQUARE_DB_CROSS,
                    (false, true, PuzzleCell::Full) => assets::SQUARE_DB_FULL,
                    (true, false, PuzzleCell::Empty) => assets::SQUARE_BD_EMPTY,
                    (true, false, PuzzleCell::Cross) => assets::SQUARE_BD_CROSS,
                    (true, false, PuzzleCell::Full) => assets::SQUARE_BD_FULL,
                    (true, true, PuzzleCell::Empty) => assets::SQUARE_BB_EMPTY,
                    (true, true, PuzzleCell::Cross) => assets::SQUARE_BB_CROSS,
                    (true, true, PuzzleCell::Full) => assets::SQUARE_BB_FULL,
                };
                let dst = (
                    (puzzle_left + col * 8) as i16,
                    (puzzle_top + row * 8) as i16,
                );
                obj_index = image.render_to_objects(obj_index, dst, STEREO);
            }
            let dst = (puzzle_right as i16, (puzzle_top + row * 8) as i16);
            obj_index = assets::SQUARE_RIGHT.render_to_objects(obj_index, dst, STEREO);
        }
        for col in 0..self.puzzle.width {
            let dst = ((puzzle_left + col * 8) as i16, puzzle_bottom as i16);
            obj_index = assets::SQUARE_BOTTOM.render_to_objects(obj_index, dst, STEREO);
        }
        obj_index = assets::SQUARE_BOTTOM_RIGHT.render_to_objects(
            obj_index,
            (puzzle_right as i16, puzzle_bottom as i16),
            STEREO,
        );

        if !self.solved {
            let cursor_x = (puzzle_left + self.cursor.0 * 8) as i16;
            let cursor_y = (puzzle_top + self.cursor.1 * 8) as i16;
            obj_index =
                assets::SQUARE_HOVER.render_to_objects(obj_index, (cursor_x, cursor_y), STEREO);
        }

        for (row, numbers) in self.row_numbers.iter().enumerate() {
            let mut num_x = (puzzle_left - 16) as i16;
            let num_y = (puzzle_top + row * 8) as i16;
            for &(num, solved) in numbers.iter().rev() {
                let image = if solved {
                    NUMBERS_DIM[num as usize - 1]
                } else {
                    NUMBERS[num as usize - 1]
                };
                obj_index = image.render_to_objects(obj_index, (num_x, num_y), STEREO);
                num_x -= 8;
            }
        }

        for (col, numbers) in self.col_numbers.iter().enumerate() {
            let num_x = (puzzle_left + col * 8) as i16;
            let mut num_y = (puzzle_top - 16) as i16;
            for &(num, solved) in numbers.iter().rev() {
                let image = if solved {
                    NUMBERS_DIM[num as usize - 1]
                } else {
                    NUMBERS[num as usize - 1]
                };
                obj_index = image.render_to_objects(obj_index, (num_x, num_y), STEREO);
                num_y -= 8;
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

    fn row_count(&self, row: usize) -> ArrayVec<(u8, bool), MAX_PUZZLE_SIZE> {
        let range_start = row * self.puzzle.width;
        let range_end = range_start + self.puzzle.width;
        let indexes = range_start..range_end;
        self.line_count(indexes)
    }

    fn col_count(&self, col: usize) -> ArrayVec<(u8, bool), MAX_PUZZLE_SIZE> {
        let range_start = col;
        let range_end = col + self.puzzle.width * self.puzzle.height;
        let indexes = (range_start..range_end).step_by(self.puzzle.width);
        self.line_count(indexes)
    }

    fn line_count(
        &self,
        indexes: impl Iterator<Item = usize>,
    ) -> ArrayVec<(u8, bool), MAX_PUZZLE_SIZE> {
        let mut cells = ArrayVec::<PuzzleCell, MAX_PUZZLE_SIZE>::new();
        let mut solution = ArrayVec::<u8, MAX_PUZZLE_SIZE>::new();
        let mut possibilities = ArrayVec::<u8, MAX_PUZZLE_SIZE>::new();
        {
            let mut consecutive = 0;
            for i in indexes {
                cells.push(self.cells[i]);
                if self.puzzle.cells[i] == 1 {
                    consecutive += 1;
                } else {
                    if consecutive > 0 {
                        solution.push(consecutive);
                        possibilities.push(0);
                    }
                    consecutive = 0;
                }
            }
            if consecutive > 0 {
                solution.push(consecutive);
                possibilities.push(0);
            }
        }
        if self.is_solved(&cells, &solution) {
            return solution.into_iter().map(|n| (n, true)).collect();
        }
        let mut i = 0usize;
        'outer: while i < cells.len() {
            let cell = cells[i];
            let is_start_of_group = matches!(cell, PuzzleCell::Full)
                && (i == 0 || matches!(cells[i - 1], PuzzleCell::Cross));
            if is_start_of_group {
                let start = i;
                loop {
                    i += 1;
                    match cells.get(i) {
                        None | Some(PuzzleCell::Cross) => {
                            break;
                        }
                        Some(PuzzleCell::Empty) => {
                            break 'outer;
                        }
                        Some(PuzzleCell::Full) => {
                            continue;
                        }
                    }
                }
                let size = (i - start) as u8;
                let mut something_is_valid = false;
                for solution_index in solution
                    .iter()
                    .enumerate()
                    .filter_map(|(i, n)| (*n == size).then_some(i))
                {
                    let is_valid = is_valid(&cells[..start], &solution[..solution_index])
                        && is_valid(&cells[i + 1..], &solution[solution_index + 1..]);
                    if is_valid {
                        something_is_valid = true;
                        possibilities[solution_index] += 1;
                    }
                }
                if !something_is_valid {
                    // couldn't find a solution for this closed group, so the player must have goofed
                    return solution.into_iter().map(|n| (n, false)).collect();
                }
            } else {
                i += 1;
            }
        }

        solution
            .into_iter()
            .zip(possibilities.into_iter().map(|n| n == 1))
            .collect()
    }

    fn is_solved(&self, mut cells: &[PuzzleCell], solution: &[u8]) -> bool {
        for count in solution {
            while let Some((PuzzleCell::Empty | PuzzleCell::Cross, rest)) = cells.split_first() {
                cells = rest;
            }
            for _ in 0..*count {
                let Some((PuzzleCell::Full, rest)) = cells.split_first() else {
                    return false;
                };
                cells = rest;
            }
            if let Some(PuzzleCell::Full) = cells.first() {
                return false;
            }
        }
        cells
            .iter()
            .all(|c| matches!(c, PuzzleCell::Empty | PuzzleCell::Cross))
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
                PuzzleCell::Cross => PuzzleCell::Empty,
                _ => PuzzleCell::Cross,
            };
            self.cursor_behavior = Some(new_cell);
        }
        if pressed.a() {
            let new_cell = match self.cells[index] {
                PuzzleCell::Full => PuzzleCell::Empty,
                _ => PuzzleCell::Full,
            };
            self.cursor_behavior = Some(new_cell);
        }
        if !held.a() && !held.b() {
            self.cursor_behavior = None;
        }
        if let Some(behavior) = self.cursor_behavior {
            self.cells[index] = behavior;
            self.col_numbers[self.cursor.0] = self.col_count(self.cursor.0);
            self.row_numbers[self.cursor.1] = self.row_count(self.cursor.1);
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
                    PuzzleCell::Empty | PuzzleCell::Cross => 0,
                    PuzzleCell::Full => 1,
                };
                expected == actual
            })
    }
}

fn is_valid(mut cells: &[PuzzleCell], solution: &[u8]) -> bool {
    let Some((&count, solution)) = solution.split_first() else {
        return cells
            .iter()
            .all(|c| matches!(c, PuzzleCell::Empty | PuzzleCell::Cross));
    };
    'outer: loop {
        while let Some((PuzzleCell::Cross, rest)) = cells.split_first() {
            cells = rest;
        }
        let old_cells = cells;
        for _ in 0..count {
            let Some((next, rest)) = cells.split_first() else {
                return false;
            };
            cells = rest;
            if matches!(next, PuzzleCell::Cross) {
                continue 'outer;
            }
        }
        match cells.split_first() {
            None => return solution.is_empty(),
            Some((PuzzleCell::Full, _)) => {}
            Some((PuzzleCell::Empty, rest)) => {
                if is_valid(rest, solution) {
                    return true;
                }
            }
            Some((PuzzleCell::Cross, rest)) => {
                if is_valid(rest, solution) {
                    return true;
                }
                // optimization: if the region we're inspecting ends with a cross,
                // there's no room for the rest
                continue;
            }
        }
        let (next, rest) = old_cells.split_first().unwrap();
        if matches!(next, PuzzleCell::Full) {
            return false;
        }
        cells = rest;
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
