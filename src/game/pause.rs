use vb_graphics::text::TextRenderer;
use vb_rt::sys::vip;

use crate::{assets, state::GameState};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Continue,
    Restart,
    Quit,
}
impl MenuItem {
    fn next(self) -> MenuItem {
        match self {
            Self::Continue => Self::Restart,
            Self::Restart => Self::Quit,
            Self::Quit => Self::Continue,
        }
    }
    fn prev(self) -> MenuItem {
        match self {
            Self::Continue => Self::Quit,
            Self::Restart => Self::Continue,
            Self::Quit => Self::Restart,
        }
    }
}

pub struct PauseMenu {
    active: MenuItem,
    continue_text: TextRenderer,
    restart_text: TextRenderer,
    quit_text: TextRenderer,
}

impl PauseMenu {
    pub fn new() -> Self {
        assets::PAUSE.render_to_bgmap(3, (0, 0));

        let active = MenuItem::Continue;

        let mut continue_text = TextRenderer::new(&assets::MENU, 758, (12, 2));
        continue_text.draw_text(b"Continue");
        continue_text.render_to_bgmap(3, (0, 32));
        let mut restart_text = TextRenderer::new(&assets::MENU, 782, (12, 2));
        restart_text.draw_text(b"Restart");
        restart_text.render_to_bgmap(3, (0, 34));
        let mut quit_text = TextRenderer::new(&assets::MENU, 806, (12, 2));
        quit_text.draw_text(b"Quit");
        quit_text.render_to_bgmap(3, (0, 36));

        Self {
            active,
            continue_text,
            restart_text,
            quit_text,
        }
    }

    pub fn init(&mut self) {
        self.active = MenuItem::Continue;
    }

    pub fn draw(&self, next_world: usize) -> usize {
        let mut next_world = next_world;
        let world = vip::WORLDS.index(next_world);
        next_world -= 1;
        world.header().write(
            vip::WorldHeader::new()
                .with_bgm(vip::WorldMode::Normal)
                .with_lon(true)
                .with_ron(true)
                .with_bg_map_base(3),
        );
        world.gx().write(140);
        world.gp().write(-4);
        world.gy().write(78);
        world.mx().write(0);
        world.my().write(0);
        world.w().write(191);
        world.h().write(111);

        for (index, (item, text)) in [
            (MenuItem::Continue, &self.continue_text),
            (MenuItem::Restart, &self.restart_text),
            (MenuItem::Quit, &self.quit_text),
        ]
        .into_iter()
        .enumerate()
        {
            let x = 158;
            let y = 86 + (index as i16 * 16);

            let world = vip::WORLDS.index(next_world);
            next_world -= 1;
            world.header().write(
                vip::WorldHeader::new()
                    .with_bgm(vip::WorldMode::Normal)
                    .with_lon(true)
                    .with_ron(true)
                    .with_bg_map_base(3),
            );
            world.gx().write(x);
            world.gp().write(if item == self.active { -6 } else { -3 });
            world.gy().write(y);
            world.mx().write(0);
            world.my().write(256 + (index as i16 * 16));
            world.w().write(text.width() - 1);
            world.h().write(15);
        }

        next_world
    }

    pub fn update(&mut self, state: &GameState) -> Option<MenuItem> {
        let held = state.directions_held();
        if held.lu() {
            self.active = self.active.prev();
        }
        if held.ld() {
            self.active = self.active.next();
        }
        let pressed = state.buttons_pressed();
        if pressed.a() {
            Some(self.active)
        } else if pressed.b() || pressed.sta() {
            Some(MenuItem::Continue)
        } else {
            None
        }
    }
}
