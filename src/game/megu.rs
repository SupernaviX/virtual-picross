use rand::Rng;
use vb_rt::sys::vip;

use crate::{assets, state::GameState};

const BG: u8 = 5;

pub struct Megu {
    back_l: (i16, i16),
    back_r: (i16, i16),
    mid_l: (i16, i16),
    mid_r: (i16, i16),
    front: (i16, i16),

    pos: (i16, i16),
    face_right: bool,
    jiggle: bool,
    counter: u8,
}

impl Megu {
    pub fn new() -> Self {
        let back_l = assets::MEGU_IDLE_BACK_L.render_to_bgmap(BG, (0, 0));
        let back_r = assets::MEGU_IDLE_BACK_R.render_to_bgmap(BG, (0, 32));
        let mid_l = assets::MEGU_IDLE_MID_L.render_to_bgmap(BG, (7, 0));
        let mid_r = assets::MEGU_IDLE_MID_R.render_to_bgmap(BG, (7, 32));
        let front = assets::MEGU_IDLE_FRONT.render_to_bgmap(BG, (11, 0));
        Self {
            back_l,
            back_r,
            mid_l,
            mid_r,
            front,
            pos: (0, 0),
            face_right: false,
            jiggle: false,
            counter: 0,
        }
    }

    pub fn init(&mut self, center: (i16, i16)) {
        self.pos = (center.0 - 24, center.1 - 40);
        self.face_right = false;
        self.jiggle = false;
        self.counter = 0;
    }

    pub fn draw(&self, next_world: usize) -> usize {
        let mut next_world = next_world;

        let back = if self.face_right {
            self.back_r
        } else {
            self.back_l
        };
        let world = vip::WORLDS.index(next_world);
        next_world -= 1;
        world.header().write(
            vip::WorldHeader::new()
                .with_bgm(vip::WorldMode::Normal)
                .with_lon(true)
                .with_ron(true)
                .with_bg_map_base(BG),
        );
        world.gx().write(self.pos.0);
        world.gp().write(2);
        world.gy().write(self.pos.1 + 16);
        world.mx().write(back.0);
        world.my().write(back.1);
        world.w().write(47);
        world.h().write(55);

        let mid = if self.face_right {
            self.mid_r
        } else {
            self.mid_l
        };
        let world = vip::WORLDS.index(next_world);
        next_world -= 1;
        world.header().write(
            vip::WorldHeader::new()
                .with_bgm(vip::WorldMode::Normal)
                .with_lon(true)
                .with_ron(true)
                .with_bg_map_base(BG),
        );
        world.gx().write(self.pos.0 + 8);
        world.gp().write(0);
        world.gy().write(self.pos.1);
        world.mx().write(mid.0);
        world.my().write(mid.1);
        world.w().write(31);
        world.h().write(79);

        let world = vip::WORLDS.index(next_world);
        next_world -= 1;
        world.header().write(
            vip::WorldHeader::new()
                .with_bgm(vip::WorldMode::Normal)
                .with_lon(true)
                .with_ron(true)
                .with_bg_map_base(BG),
        );
        world.gx().write(self.pos.0 + 8);
        world.gp().write(-1);
        world
            .gy()
            .write(self.pos.1 + if self.jiggle { 23 } else { 24 });
        world.mx().write(self.front.0);
        world.my().write(self.front.1);
        world.w().write(31);
        world.h().write(15);

        next_world
    }

    pub fn update(&mut self, state: &mut GameState) {
        self.counter += 1;
        if self.counter % 4 == 0 {
            let rand = state.rand().random_range(0..16);
            self.jiggle = !self.jiggle && rand == 0;
            if self.counter == 16 {
                self.face_right = rand < 4;
                self.counter = 0;
            }
        }
    }
}
