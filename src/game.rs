use core::fmt::Write;

use arrayvec::ArrayVec;
use vb_graphics::{
    Image,
    text::{BufferedTextRenderer, TextRenderer},
};
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

enum Zoom {
    One,
    Two,
}
impl Zoom {
    fn cell_pixels(&self) -> usize {
        match self {
            Self::One => 8,
            Self::Two => 16,
        }
    }
}

enum PuzzleState {
    Playing,
    Moving,
    ShowingText,
}

const MAX_PUZZLE_SIZE: usize = 20;
const TEXT_TOP: usize = 184;

pub struct Game {
    puzzle: &'static Puzzle,
    cells: [PuzzleCell; MAX_PUZZLE_SIZE * MAX_PUZZLE_SIZE],
    row_numbers: ArrayVec<ArrayVec<(u8, bool), MAX_PUZZLE_SIZE>, MAX_PUZZLE_SIZE>,
    col_numbers: ArrayVec<ArrayVec<(u8, bool), MAX_PUZZLE_SIZE>, MAX_PUZZLE_SIZE>,
    puzzle_pos: (usize, usize),
    zoom: Zoom,
    cursor: (usize, usize),
    cursor_behavior: Option<PuzzleCell>,
    cursor_delay: u8,
    state: PuzzleState,
    timer: u32,
    timer_text: TextRenderer,
    name_text: BufferedTextRenderer<64>,
    source_text: BufferedTextRenderer<64>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            puzzle: &EMPTY,
            cells: [PuzzleCell::Empty; MAX_PUZZLE_SIZE * MAX_PUZZLE_SIZE],
            row_numbers: ArrayVec::new(),
            col_numbers: ArrayVec::new(),
            puzzle_pos: (192, 112),
            zoom: Zoom::One,
            cursor: (0, 0),
            cursor_behavior: None,
            cursor_delay: 0,
            state: PuzzleState::Playing,
            timer: 0,
            timer_text: TextRenderer::new(&assets::MENU, 400, (12, 2)),
            name_text: TextRenderer::new(&assets::MENU, 256, (24, 3)).buffered(3),
            source_text: TextRenderer::new(&assets::MENU, 328, (24, 3)).buffered(2),
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

        let (width_cells, height_cells) = self.size_cells();
        self.zoom = if width_cells >= 24 || height_cells >= 14 {
            Zoom::One
        } else {
            Zoom::Two
        };
        let cell_pixels = self.zoom.cell_pixels();
        let x_offset = (384 - width_cells * cell_pixels) / 2;
        let y_offset = (224 - height_cells * cell_pixels) / 2;

        let puzzle_right = 384 - cell_pixels - x_offset;
        let puzzle_bottom = 224 - cell_pixels - y_offset;
        let puzzle_left = puzzle_right - (self.puzzle.width * cell_pixels);
        let puzzle_top = puzzle_bottom - (self.puzzle.height * cell_pixels);
        self.puzzle_pos = (puzzle_left, puzzle_top);

        self.cursor = (0, 0);
        self.cursor_behavior = None;
        self.cursor_delay = 0;
        self.state = PuzzleState::Playing;
        self.timer = 0;
        self.timer_text.clear();
        let _ = write!(&mut self.timer_text, "00:00:00");
        self.timer_text.render_to_bgmap(1, (0, 0));
        self.name_text.clear();
        let _ = self.name_text.draw_text(self.puzzle.name);
        self.name_text.render_to_bgmap(1, (0, 32));
        self.source_text.clear();
        let _ = self.source_text.draw_text(self.puzzle.source);
        self.source_text.render_to_bgmap(1, (0, 48));
    }

    pub fn size_cells(&self) -> (usize, usize) {
        let width = self.puzzle.width
            + 1
            + self
                .row_numbers
                .iter()
                .map(|n| n.len())
                .max()
                .unwrap_or_default();
        let height = self.puzzle.height
            + 1
            + self
                .col_numbers
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

        let (puzzle_left, puzzle_top) = self.puzzle_pos;
        let (cell_pixels, game_assets) = match self.zoom {
            Zoom::One => (8, GameAssets(&GAME_ASSETS_1X)),
            Zoom::Two => (16, GameAssets(&GAME_ASSETS_2X)),
        };

        let puzzle_right = puzzle_left + (self.puzzle.width * cell_pixels);
        let puzzle_bottom = puzzle_top + (self.puzzle.height * cell_pixels);

        for row in 0..self.puzzle.height {
            for col in 0..self.puzzle.width {
                let col_bright = col > 0 && col % 5 == 0;
                let row_bright = row > 0 && row % 5 == 0;
                if let PuzzleState::ShowingText = self.state {
                    let answer = self.puzzle.cells[row * self.puzzle.width + col];
                    if answer == 1 {
                        let image = game_assets.square_final();
                        let dst = (
                            (puzzle_left + col * cell_pixels) as i16,
                            (puzzle_top + row * cell_pixels) as i16,
                        );
                        obj_index = image.render_to_objects(obj_index, dst, STEREO);
                    }
                } else {
                    let cell = self.cells[row * self.puzzle.width + col];
                    let image = game_assets.square(col_bright, row_bright, cell);
                    let dst = (
                        (puzzle_left + col * cell_pixels) as i16,
                        (puzzle_top + row * cell_pixels) as i16,
                    );
                    obj_index = image.render_to_objects(obj_index, dst, STEREO);
                }
            }
            if let PuzzleState::Playing | PuzzleState::Moving = self.state {
                let dst = (puzzle_right as i16, (puzzle_top + row * cell_pixels) as i16);
                obj_index = game_assets
                    .square_right()
                    .render_to_objects(obj_index, dst, STEREO);
            }
        }
        if let PuzzleState::Playing | PuzzleState::Moving = self.state {
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
        }

        if let PuzzleState::Playing = self.state {
            let cursor_x = (puzzle_left + self.cursor.0 * cell_pixels) as i16;
            let cursor_y = (puzzle_top + self.cursor.1 * cell_pixels) as i16;
            obj_index = game_assets.square_hover().render_to_objects(
                obj_index,
                (cursor_x, cursor_y),
                STEREO,
            );

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
        world.w().write(self.timer_text.width() - 1);
        world.h().write(20);

        if let PuzzleState::ShowingText = self.state {
            let world = vip::WORLDS.index(next_world);
            next_world -= 1;

            let text_width = self.name_text.final_width();
            let text_height = assets::MENU.line_height as i16;

            world.header().write(
                vip::WorldHeader::new()
                    .with_bgm(vip::WorldMode::Normal)
                    .with_lon(true)
                    .with_ron(true)
                    .with_bg_map_base(1),
            );
            world.gx().write(192 - text_width / 2);
            world.gp().write(0);
            world.gy().write(TEXT_TOP as i16);
            world.mx().write(0);
            world.my().write(256);
            world.w().write(self.name_text.width() - 1);
            world.h().write(text_height - 1);

            let world = vip::WORLDS.index(next_world);
            next_world -= 1;

            let text_width = self.source_text.final_width();
            let text_height = assets::MENU.line_height as i16;

            world.header().write(
                vip::WorldHeader::new()
                    .with_bgm(vip::WorldMode::Normal)
                    .with_lon(true)
                    .with_ron(true)
                    .with_bg_map_base(1),
            );
            world.gx().write(192 - text_width / 2);
            world.gp().write(0);
            world.gy().write(TEXT_TOP as i16 + text_height);
            world.mx().write(0);
            world.my().write(384);
            world.w().write(self.source_text.width() - 1);
            world.h().write(text_height - 1);
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
                let mut unique_possible_position = None;
                for solution_index in solution
                    .iter()
                    .enumerate()
                    .filter_map(|(i, n)| (*n == size).then_some(i))
                {
                    let cells_before = &cells[..start];
                    let solution_before = &solution[..solution_index];
                    let cells_after = if i < cells.len() {
                        &cells[i + 1..]
                    } else {
                        &[]
                    };
                    let solution_after = if solution_index < solution.len() {
                        &solution[solution_index + 1..]
                    } else {
                        &[]
                    };
                    let is_valid = is_valid(cells_before, solution_before)
                        && is_valid(cells_after, solution_after);
                    if is_valid {
                        if !something_is_valid {
                            something_is_valid = true;
                            unique_possible_position = Some(solution_index);
                        } else {
                            unique_possible_position = None;
                        }
                    }
                }
                if !something_is_valid {
                    // couldn't find a solution for this closed group, so the player must have goofed
                    return solution.into_iter().map(|n| (n, false)).collect();
                }
                if let Some(solution_index) = unique_possible_position {
                    possibilities[solution_index] += 1;
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
        if let PuzzleState::ShowingText = self.state {
            if self.name_text.update() {
                self.source_text.update();
            }
            let pressed = state.buttons_pressed();
            return (pressed.a() || pressed.sta()).then_some(self.timer);
        }
        if let PuzzleState::Moving = self.state {
            let target_puzzle_left = (384 - self.puzzle.width * self.zoom.cell_pixels()) / 2;
            let target_puzzle_top = (TEXT_TOP - self.puzzle.height * self.zoom.cell_pixels()) / 2;
            if self.puzzle_pos.0 > target_puzzle_left {
                self.puzzle_pos.0 -= 1;
            } else if self.puzzle_pos.1 > target_puzzle_top {
                self.puzzle_pos.1 -= 1;
            } else {
                self.state = PuzzleState::ShowingText;
            }
            return None;
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
                self.state = PuzzleState::Moving;
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
        let mut full_seen = false;
        for _ in 0..count {
            let Some((next, rest)) = cells.split_first() else {
                return false;
            };
            cells = rest;
            match next {
                PuzzleCell::Empty => {}
                PuzzleCell::Full => {
                    full_seen = true;
                }
                PuzzleCell::Cross => {
                    if full_seen {
                        // found a group of cells not big enough to be the next group we need
                        return false;
                    } else {
                        continue 'outer;
                    }
                }
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
                if full_seen {
                    // if we saw a full cell in a region ending with a cross,
                    // and the pattern wasn't valid, there's def no way to correct that
                    return false;
                }
                continue;
            }
        }
        let (next, rest) = old_cells.split_first().unwrap();
        if matches!(next, PuzzleCell::Full) {
            // If this region started with a full cell and we couldn't make it valid,
            // the whole row must be invalid
            return false;
        }
        // try filling the same pattern shifted once cell to the rust
        cells = rest;
    }
}

struct GameAssets(&'static [&'static Image; 57]);
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
    fn square_final(&self) -> &'static Image {
        self.0[16]
    }

    fn number(&self, num: u8) -> &'static Image {
        self.0[num as usize + 16]
    }
    fn number_dim(&self, num: u8) -> &'static Image {
        self.0[num as usize + 36]
    }
}

const GAME_ASSETS_1X: [&Image; 57] = [
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
    &assets::SQUARE_FINAL,
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

const GAME_ASSETS_2X: [&Image; 57] = [
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
    &assets::SQUARE_FINAL_2X,
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
