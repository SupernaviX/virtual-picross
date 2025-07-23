use vb_rt::sys::vip;

use crate::{assets, state::GameState};

const BG: u8 = 4;
pub struct Title {
    timer: u8,
}

impl Title {
    pub fn new() -> Self {
        assets::TITLE_LEFT.render_to_bgmap(BG, (0, 0));
        assets::TITLE_RIGHT.render_to_bgmap(BG, (0, 28));
        assets::START.render_to_bgmap(BG, (0, 56));
        Self { timer: 0 }
    }

    pub fn draw(&self) {
        let mut next_world = 31;

        let world = vip::WORLDS.index(next_world);
        next_world -= 1;
        world.header().write(
            vip::WorldHeader::new()
                .with_bgm(vip::WorldMode::Normal)
                .with_lon(true)
                .with_ron(false)
                .with_bg_map_base(BG),
        );
        world.gx().write(0);
        world.gp().write(0);
        world.gy().write(0);
        world.mx().write(0);
        world.my().write(0);
        world.w().write(383);
        world.h().write(223);

        let world = vip::WORLDS.index(next_world);
        next_world -= 1;
        world.header().write(
            vip::WorldHeader::new()
                .with_bgm(vip::WorldMode::Normal)
                .with_lon(false)
                .with_ron(true)
                .with_bg_map_base(BG),
        );
        world.gx().write(0);
        world.gp().write(0);
        world.gy().write(0);
        world.mx().write(0);
        world.my().write(224);
        world.w().write(383);
        world.h().write(223);

        if self.timer / 32 == 0 {
            let world = vip::WORLDS.index(next_world);
            next_world -= 1;
            world.header().write(
                vip::WorldHeader::new()
                    .with_bgm(vip::WorldMode::Normal)
                    .with_lon(true)
                    .with_ron(true)
                    .with_bg_map_base(BG),
            );
            world.gx().write(128);
            world.gp().write(-4);
            world.gy().write(168);
            world.mx().write(0);
            world.my().write(448);
            world.w().write(127);
            world.h().write(31);
        }

        let world = vip::WORLDS.index(next_world);
        world.header().write(vip::WorldHeader::new().with_end(true));
    }

    pub fn update(&mut self, state: &GameState) -> bool {
        let pressed = state.buttons_pressed();
        if pressed.sta() {
            return true;
        }
        self.timer = (self.timer + 1) % 64;
        false
    }
}
