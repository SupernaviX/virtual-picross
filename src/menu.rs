use core::fmt::Write as _;

use vb_graphics::text::{BufferedTextRenderer, TextRenderer};
use vb_rt::sys::vip;

use crate::{
    assets,
    game::GameResult,
    puzzle::{ICONS, PUZZLES, Puzzle},
    save,
    state::GameState,
};

const BG: u8 = 2;

pub struct Menu {
    index: usize,
    cursor_delay: u8,
    times: [Option<u32>; PUZZLES.len()],
    index_renderer: TextRenderer,
    size_renderer: TextRenderer,
    name_renderer: BufferedTextRenderer<32>,
    time_renderer: TextRenderer,
}

impl Menu {
    pub fn new() -> Self {
        let index_renderer = TextRenderer::new(&assets::MENU, 512, (12, 2));
        index_renderer.render_to_bgmap(BG, (0, 0));
        let size_renderer = TextRenderer::new(&assets::MENU, 536, (12, 2));
        size_renderer.render_to_bgmap(BG, (0, 2));
        let name_renderer = TextRenderer::new(&assets::MENU, 560, (28, 3));
        name_renderer.render_to_bgmap(BG, (0, 4));
        let time_renderer = TextRenderer::new(&assets::MENU, 644, (20, 2));
        time_renderer.render_to_bgmap(BG, (0, 6));
        for index in 0..PUZZLES.len() {
            let row = (index / 5) as u8;
            let col = (index % 5) as u8;
            let dst = (col * 5, (row * 5) + 32);
            let index = (5 * row + col) as usize;
            ICONS[index].render_to_bgmap(BG, dst);
        }
        let mut me = Self {
            index: 0,
            cursor_delay: 0,
            times: save::load_times(),
            index_renderer,
            size_renderer,
            name_renderer: name_renderer.buffered(2),
            time_renderer,
        };
        me.display_stats();
        me
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

        let page = self.index / 15;

        for index in (page * 15)..((page + 1) * 15).min(PUZZLES.len()) {
            let index_on_page = index % 15;
            let (row, col) = (index_on_page / 5, index_on_page % 5);
            let dst = (52 + col as i16 * 56, 8 + row as i16 * 56);
            let (menu_item, stereo) = if index == self.index {
                (assets::MENU_ITEM_SELECTED, STEREO.with_jp(-4))
            } else {
                (assets::MENU_ITEM, STEREO)
            };

            if self.times[index].is_some() {
                let world = vip::WORLDS.index(next_world);
                next_world -= 1;
                world.header().write(
                    vip::WorldHeader::new()
                        .with_bgm(vip::WorldMode::Normal)
                        .with_lon(true)
                        .with_ron(true)
                        .with_bg_map_base(BG),
                );
                world.gx().write(dst.0 + 8);
                world.gp().write(if index == self.index { -4 } else { 0 });
                world.gy().write(dst.1 + 8);
                world.mx().write((index as i16 % 5) * 40);
                world.my().write(256 + (index as i16 / 5) * 40);
                world.w().write(40);
                world.h().write(40);
            }

            obj_index = menu_item.render_to_objects(obj_index, dst, stereo);
        }
        vip::SPT2.write(obj_index);

        let text_height = assets::MENU.line_height as i16;
        if !self.index_renderer.is_empty() {
            let world = vip::WORLDS.index(next_world);
            next_world -= 1;
            world.header().write(
                vip::WorldHeader::new()
                    .with_bgm(vip::WorldMode::Normal)
                    .with_lon(true)
                    .with_ron(true)
                    .with_bg_map_base(BG),
            );
            world.gx().write(8);
            world.gy().write(184);
            world.mx().write(0);
            world.my().write(0);
            world.w().write(self.index_renderer.width() - 1);
            world.h().write(text_height - 1);
        }

        if !self.size_renderer.is_empty() {
            let world = vip::WORLDS.index(next_world);
            next_world -= 1;
            world.header().write(
                vip::WorldHeader::new()
                    .with_bgm(vip::WorldMode::Normal)
                    .with_lon(true)
                    .with_ron(true)
                    .with_bg_map_base(BG),
            );
            world.gx().write(8);
            world.gy().write(184 + text_height);
            world.mx().write(0);
            world.my().write(16);
            world.w().write(self.size_renderer.width() - 1);
            world.h().write(text_height - 1);
        }
        if !self.name_renderer.is_empty() {
            let world = vip::WORLDS.index(next_world);
            next_world -= 1;
            world.header().write(
                vip::WorldHeader::new()
                    .with_bgm(vip::WorldMode::Normal)
                    .with_lon(true)
                    .with_ron(true)
                    .with_bg_map_base(BG),
            );
            world.gx().write(104);
            world.gy().write(184);
            world.mx().write(0);
            world.my().write(32);
            world.w().write(self.name_renderer.width() - 1);
            world.h().write(text_height - 1);
        }

        if !self.time_renderer.is_empty() {
            let world = vip::WORLDS.index(next_world);
            next_world -= 1;
            world.header().write(
                vip::WorldHeader::new()
                    .with_bgm(vip::WorldMode::Normal)
                    .with_lon(true)
                    .with_ron(true)
                    .with_bg_map_base(BG),
            );
            world.gx().write(104);
            world.gy().write(184 + text_height);
            world.mx().write(0);
            world.my().write(48);
            world.w().write(self.time_renderer.width() - 1);
            world.h().write(text_height - 1);
        }

        let world = vip::WORLDS.index(next_world);
        world.header().write(vip::WorldHeader::new().with_end(true));
    }

    pub fn update(&mut self, state: &GameState) -> Option<&'static Puzzle> {
        self.name_renderer.update();

        let pressed = state.buttons_pressed();
        if pressed.a() {
            return PUZZLES.get(self.index);
        }
        let held = state.directions_held();
        let mut cursor_moved = false;
        if held.ll() && self.index > 0 && self.cursor_delay == 0 {
            self.index -= 1;
            cursor_moved = true;
        }
        if held.lr() && self.index < PUZZLES.len() - 1 && self.cursor_delay == 0 {
            self.index += 1;
            cursor_moved = true;
        }
        if held.lu() && self.index > 4 && self.cursor_delay == 0 {
            self.index -= 5;
            cursor_moved = true;
        }
        if held.ld() && self.index < PUZZLES.len() - 1 && self.cursor_delay == 0 {
            self.index = (self.index + 5).min(PUZZLES.len() - 1);
            cursor_moved = true;
        }
        if cursor_moved {
            self.cursor_delay = 4;
            self.display_stats();
        } else {
            self.cursor_delay = self.cursor_delay.saturating_sub(1);
        }
        None
    }

    pub fn finish_puzzle(&mut self, result: GameResult) {
        if let GameResult::Won(time) = result {
            if self.times[self.index].is_none_or(|t| t > time) {
                self.times[self.index] = Some(time);
                save::save_time(self.index, time);
            }
        }
        self.display_stats();
    }

    fn display_stats(&mut self) {
        let puzzle = &PUZZLES[self.index];
        let (done, seconds) = match self.times[self.index] {
            Some(time) => (true, time / 50),
            None => (false, 0),
        };

        self.index_renderer.clear();
        let _ = write!(&mut self.index_renderer, "id: {}", self.index + 1);

        self.size_renderer.clear();
        let _ = write!(
            &mut self.size_renderer,
            "size: {}x{}",
            puzzle.width, puzzle.height
        );

        self.name_renderer.clear();
        let _ = write!(&mut self.name_renderer.inner, "title: ");
        if done {
            self.name_renderer.draw_text(puzzle.name);
        }

        self.time_renderer.clear();
        let _ = write!(&mut self.time_renderer, "time: ");
        if done {
            let _ = write!(
                &mut self.time_renderer,
                "{:02}:{:02}:{:02}",
                seconds / 60 / 60,
                (seconds / 60) % 60,
                seconds % 60,
            );
        }
    }
}
