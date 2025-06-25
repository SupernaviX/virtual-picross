use core::fmt::Write as _;

use vb_graphics::text::TextRenderer;
use vb_rt::sys::vip;

use crate::{
    assets,
    puzzle::{PUZZLES, Puzzle},
    state::GameState,
};

const BG: u8 = 2;

pub struct Menu {
    index: usize,
    cursor_delay: u8,
    renderer: TextRenderer,
}

impl Menu {
    pub fn new() -> Self {
        let renderer = TextRenderer::new(&assets::MENU, 512, (48, 10));
        renderer.render_to_bgmap(BG, (0, 0));
        let mut me = Self {
            index: 0,
            cursor_delay: 0,
            renderer,
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

        for (index, _puzzle) in PUZZLES.iter().take(15).enumerate() {
            let (row, col) = (index / 5, index % 5);
            let dst = (52 + col as i16 * 56, 8 + row as i16 * 56);
            let (menu_item, stereo) = if index == self.index {
                (assets::MENU_ITEM_SELECTED, STEREO.with_jp(-4))
            } else {
                (assets::MENU_ITEM, STEREO)
            };
            obj_index = menu_item.render_to_objects(obj_index, dst, stereo);
        }

        vip::SPT2.write(obj_index);
        if !self.renderer.is_empty() {
            let text_height = assets::MENU.line_height as i16;

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
            world.w().write(96);
            world.h().write(text_height);

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
            world.my().write(text_height);
            world.w().write(96);
            world.h().write(text_height);

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
            world.my().write(text_height * 2);
            world.w().write(287);
            world.h().write(text_height);

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
            world.my().write(text_height * 3);
            world.w().write(287);
            world.h().write(text_height);
        }

        let world = vip::WORLDS.index(next_world);
        world.header().write(vip::WorldHeader::new().with_end(true));
    }

    pub fn update(&mut self, state: &GameState) -> Option<&'static Puzzle> {
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
        if held.ld() && self.index < PUZZLES.len() - 5 && self.cursor_delay == 0 {
            self.index += 5;
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

    fn display_stats(&mut self) {
        let puzzle = &PUZZLES[self.index];

        self.renderer.clear();
        let _ = writeln!(&mut self.renderer, "id: {}", self.index + 1);
        let _ = writeln!(
            &mut self.renderer,
            "size: {}x{}",
            puzzle.width, puzzle.height
        );
        let _ = write!(&mut self.renderer, "title: ");
        self.renderer.draw_text(puzzle.name);
        let _ = write!(&mut self.renderer, "\ntime: 00:00:00");
    }
}
