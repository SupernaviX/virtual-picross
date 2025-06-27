use core::fmt::Write;

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
    solved: bool,
    timer: u32,
    timer_text: TextRenderer,
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
            solved: false,
            timer: 0,
            timer_text: TextRenderer::new(&assets::MENU, 640, (12, 2)),
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
        self.solved = false;
        self.timer = 0;
        self.timer_text.clear();
        let _ = write!(&mut self.timer_text, "00:00:00");
        self.timer_text.render_to_bgmap(1, (0, 0));
    }

    pub fn size_cells(&self) -> (usize, usize) {
        let width = self.puzzle.width
            + 1
            + self
                .col_numbers
                .iter()
                .map(|n| n.len())
                .max()
                .unwrap_or_default();
        let height = self.puzzle.height
            + 1
            + self
                .row_numbers
                .iter()
                .map(|n| n.len())
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

        let (width_cells, height_cells) = self.size_cells();
        let (cell_pixels, game_assets) = if width_cells >= 24 || height_cells >= 14 {
            (8, GameAssets(&GAME_ASSETS_1X))
        } else {
            (16, GameAssets(&GAME_ASSETS_2X))
        };
        let x_offset = (384 - width_cells * cell_pixels) / 2;
        let y_offset = (224 - height_cells * cell_pixels) / 2;

        let puzzle_right = 384 - cell_pixels - x_offset;
        let puzzle_bottom = 224 - cell_pixels - y_offset;
        let puzzle_left = puzzle_right - (self.puzzle.width * cell_pixels);
        let puzzle_top = puzzle_bottom - (self.puzzle.height * cell_pixels);

        for row in 0..self.puzzle.height {
            for col in 0..self.puzzle.width {
                let col_bright = col > 0 && col % 5 == 0;
                let row_bright = row > 0 && row % 5 == 0;
                let cell = self.cells[row * self.puzzle.width + col];
                let image = game_assets.square(col_bright, row_bright, cell);
                let dst = (
                    (puzzle_left + col * cell_pixels) as i16,
                    (puzzle_top + row * cell_pixels) as i16,
                );
                obj_index = image.render_to_objects(obj_index, dst, STEREO);
            }
            let dst = (puzzle_right as i16, (puzzle_top + row * cell_pixels) as i16);
            obj_index = game_assets
                .square_right()
                .render_to_objects(obj_index, dst, STEREO);
        }
        for col in 0..self.puzzle.width {
            let dst = (
                (puzzle_left + col * cell_pixels) as i16,
                puzzle_bottom as i16,
            );
            obj_index = game_assets
                .square_bottom()
                .render_to_objects(obj_index, dst, STEREO);
        }
        obj_index = game_assets.square_bottom_right().render_to_objects(
            obj_index,
            (puzzle_right as i16, puzzle_bottom as i16),
            STEREO,
        );

        if !self.solved {
            let cursor_x = (puzzle_left + self.cursor.0 * cell_pixels) as i16;
            let cursor_y = (puzzle_top + self.cursor.1 * cell_pixels) as i16;
            obj_index = game_assets.square_hover().render_to_objects(
                obj_index,
                (cursor_x, cursor_y),
                STEREO,
            );
        }

        for (row, numbers) in self.row_numbers.iter().enumerate() {
            let mut num_x = (puzzle_left - 2 * cell_pixels) as i16;
            let num_y = (puzzle_top + row * cell_pixels) as i16;
            for &(num, solved) in numbers.iter().rev() {
                let image = if solved {
                    game_assets.number_dim(num)
                } else {
                    game_assets.number(num)
                };
                obj_index = image.render_to_objects(obj_index, (num_x, num_y), STEREO);
                num_x -= cell_pixels as i16;
            }
        }

        for (col, numbers) in self.col_numbers.iter().enumerate() {
            let num_x = (puzzle_left + col * cell_pixels) as i16;
            let mut num_y = (puzzle_top - 2 * cell_pixels) as i16;
            for &(num, solved) in numbers.iter().rev() {
                let image = if solved {
                    game_assets.number_dim(num)
                } else {
                    game_assets.number(num)
                };
                obj_index = image.render_to_objects(obj_index, (num_x, num_y), STEREO);
                num_y -= cell_pixels as i16;
            }
        }

        vip::SPT2.write(obj_index);

        let world = vip::WORLDS.index(next_world);
        next_world -= 1;
        world.header().write(
            vip::WorldHeader::new()
                .with_bgm(vip::WorldMode::Normal)
                .with_lon(true)
                .with_ron(true)
                .with_bg_map_base(1),
        );
        world.gx().write(8);
        world.gp().write(0);
        world.gy().write(8);
        world.mx().write(0);
        world.my().write(0);
        world.w().write(100);
        world.h().write(20);

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
            world.gp().write(0);
            world.gy().write(112 - text_height / 2);
            world.mx().write(0);
            world.my().write(256);
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
                            continue 'outer;
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

    pub fn update(&mut self, state: &GameState) -> Option<u32> {
        if self.solved {
            let pressed = state.buttons_pressed();
            return (pressed.a() || pressed.sta()).then_some(self.timer);
        }
        self.timer += 1;
        if self.timer % 50 == 0 {
            let seconds = self.timer / 50;
            let minutes = (seconds / 60) % 60;
            let hours = seconds / 60 / 60;

            self.timer_text.clear();
            let _ = write!(
                &mut self.timer_text,
                "{:02}:{:02}:{:02}",
                hours,
                minutes,
                seconds % 60
            );
        }

        let held = state.directions_held();
        let mut cursor_moved = false;
        let mut handle_move = |button: bool, delta: (isize, isize)| {
            if button && self.cursor_delay == 0 {
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

        handle_move(held.ll(), (-1, 0));
        handle_move(held.lr(), (1, 0));
        handle_move(held.lu(), (0, -1));
        handle_move(held.ld(), (0, 1));

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
        let held = state.buttons_held();
        if !held.a() && !held.b() {
            self.cursor_behavior = None;
        }
        if let Some(behavior) = self.cursor_behavior {
            self.cells[index] = behavior;
            self.col_numbers[self.cursor.0] = self.col_count(self.cursor.0);
            self.row_numbers[self.cursor.1] = self.row_count(self.cursor.1);
            if self.has_been_solved() {
                let mut renderer = TextRenderer::new(&assets::VIRTUAL_BOY, 256, (24, 14));
                renderer.draw_text(self.puzzle.name);
                renderer.render_to_bgmap(1, (0, 32));
                self.solved = true;
            }
        }
        None
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

struct GameAssets(&'static [&'static Image; 56]);
impl GameAssets {
    fn square(&self, col_bright: bool, row_bright: bool, cell: PuzzleCell) -> &'static Image {
        let index = match (col_bright, row_bright, cell) {
            (false, false, PuzzleCell::Empty) => 0,
            (false, false, PuzzleCell::Cross) => 1,
            (false, false, PuzzleCell::Full) => 2,
            (false, true, PuzzleCell::Empty) => 3,
            (false, true, PuzzleCell::Cross) => 4,
            (false, true, PuzzleCell::Full) => 5,
            (true, false, PuzzleCell::Empty) => 6,
            (true, false, PuzzleCell::Cross) => 7,
            (true, false, PuzzleCell::Full) => 8,
            (true, true, PuzzleCell::Empty) => 9,
            (true, true, PuzzleCell::Cross) => 10,
            (true, true, PuzzleCell::Full) => 11,
        };
        self.0[index]
    }
    fn square_right(&self) -> &'static Image {
        self.0[12]
    }
    fn square_bottom(&self) -> &'static Image {
        self.0[13]
    }
    fn square_bottom_right(&self) -> &'static Image {
        self.0[14]
    }
    fn square_hover(&self) -> &'static Image {
        self.0[15]
    }

    fn number(&self, num: u8) -> &'static Image {
        self.0[num as usize + 15]
    }
    fn number_dim(&self, num: u8) -> &'static Image {
        self.0[num as usize + 35]
    }
}

const GAME_ASSETS_1X: [&Image; 56] = [
    &assets::SQUARE_DD_EMPTY,
    &assets::SQUARE_DD_CROSS,
    &assets::SQUARE_DD_FULL,
    &assets::SQUARE_DB_EMPTY,
    &assets::SQUARE_DB_CROSS,
    &assets::SQUARE_DB_FULL,
    &assets::SQUARE_BD_EMPTY,
    &assets::SQUARE_BD_CROSS,
    &assets::SQUARE_BD_FULL,
    &assets::SQUARE_BB_EMPTY,
    &assets::SQUARE_BB_CROSS,
    &assets::SQUARE_BB_FULL,
    &assets::SQUARE_RIGHT,
    &assets::SQUARE_BOTTOM,
    &assets::SQUARE_BOTTOM_RIGHT,
    &assets::SQUARE_HOVER,
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

const GAME_ASSETS_2X: [&Image; 56] = [
    &assets::SQUARE_DD_EMPTY_2X,
    &assets::SQUARE_DD_CROSS_2X,
    &assets::SQUARE_DD_FULL_2X,
    &assets::SQUARE_DB_EMPTY_2X,
    &assets::SQUARE_DB_CROSS_2X,
    &assets::SQUARE_DB_FULL_2X,
    &assets::SQUARE_BD_EMPTY_2X,
    &assets::SQUARE_BD_CROSS_2X,
    &assets::SQUARE_BD_FULL_2X,
    &assets::SQUARE_BB_EMPTY_2X,
    &assets::SQUARE_BB_CROSS_2X,
    &assets::SQUARE_BB_FULL_2X,
    &assets::SQUARE_RIGHT_2X,
    &assets::SQUARE_BOTTOM_2X,
    &assets::SQUARE_BOTTOM_RIGHT_2X,
    &assets::SQUARE_HOVER_2X,
    &assets::NUMBER_1_2X,
    &assets::NUMBER_2_2X,
    &assets::NUMBER_3_2X,
    &assets::NUMBER_4_2X,
    &assets::NUMBER_5_2X,
    &assets::NUMBER_6_2X,
    &assets::NUMBER_7_2X,
    &assets::NUMBER_8_2X,
    &assets::NUMBER_9_2X,
    &assets::NUMBER_10_2X,
    &assets::NUMBER_11_2X,
    &assets::NUMBER_12_2X,
    &assets::NUMBER_13_2X,
    &assets::NUMBER_14_2X,
    &assets::NUMBER_15_2X,
    &assets::NUMBER_16_2X,
    &assets::NUMBER_17_2X,
    &assets::NUMBER_18_2X,
    &assets::NUMBER_19_2X,
    &assets::NUMBER_20_2X,
    &assets::NUMBER_1_DIM_2X,
    &assets::NUMBER_2_DIM_2X,
    &assets::NUMBER_3_DIM_2X,
    &assets::NUMBER_4_DIM_2X,
    &assets::NUMBER_5_DIM_2X,
    &assets::NUMBER_6_DIM_2X,
    &assets::NUMBER_7_DIM_2X,
    &assets::NUMBER_8_DIM_2X,
    &assets::NUMBER_9_DIM_2X,
    &assets::NUMBER_10_DIM_2X,
    &assets::NUMBER_11_DIM_2X,
    &assets::NUMBER_12_DIM_2X,
    &assets::NUMBER_13_DIM_2X,
    &assets::NUMBER_14_DIM_2X,
    &assets::NUMBER_15_DIM_2X,
    &assets::NUMBER_16_DIM_2X,
    &assets::NUMBER_17_DIM_2X,
    &assets::NUMBER_18_DIM_2X,
    &assets::NUMBER_19_DIM_2X,
    &assets::NUMBER_20_DIM_2X,
];
