use rand::Rng;
use vb_rt::sys::vip;

use crate::{assets, state::GameState};

const BG: u8 = 5;

struct Pose {
    back_l: (i16, i16),
    back_r: (i16, i16),
    mid_l: (i16, i16),
    mid_r: (i16, i16),
    front_l: (i16, i16),
    front_r: (i16, i16),
}

impl Pose {
    fn draw(&self, pos: (i16, i16), face_right: bool, jiggle: bool, next_world: usize) -> usize {
        let mut next_world = next_world;

        let back = if face_right { self.back_r } else { self.back_l };
        let world = vip::WORLDS.index(next_world);
        next_world -= 1;
        world.header().write(
            vip::WorldHeader::new()
                .with_bgm(vip::WorldMode::Normal)
                .with_lon(true)
                .with_ron(true)
                .with_bg_map_base(BG),
        );
        world.gx().write(pos.0);
        world.gp().write(2);
        world.gy().write(pos.1 + 16);
        world.mx().write(back.0);
        world.my().write(back.1);
        world.w().write(63);
        world.h().write(55);

        let mid = if face_right { self.mid_r } else { self.mid_l };
        let world = vip::WORLDS.index(next_world);
        next_world -= 1;
        world.header().write(
            vip::WorldHeader::new()
                .with_bgm(vip::WorldMode::Normal)
                .with_lon(true)
                .with_ron(true)
                .with_bg_map_base(BG),
        );
        world.gx().write(pos.0);
        world.gp().write(0);
        world.gy().write(pos.1);
        world.mx().write(mid.0);
        world.my().write(mid.1);
        world.w().write(63);
        world.h().write(79);

        let front = if face_right {
            self.front_r
        } else {
            self.front_l
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
        world.gx().write(pos.0);
        world.gp().write(-1);
        world.gy().write(pos.1 + if jiggle { 15 } else { 16 });
        world.mx().write(front.0);
        world.my().write(front.1);
        world.w().write(63);
        world.h().write(63);

        next_world
    }
}

pub struct Megu {
    idle: Pose,
    celebrating_1: Pose,
    celebrating_2: Pose,

    pos: (i16, i16),
    face_right: bool,
    jiggle: bool,
    celebrate: Option<bool>,
    counter: u8,
}

impl Megu {
    pub fn new() -> Self {
        let idle = {
            let back_l = assets::MEGU_IDLE_BACK_L.render_to_bgmap(BG, (0, 0));
            let back_r = assets::MEGU_IDLE_BACK_R.render_to_bgmap(BG, (0, 10));
            let mid_l = assets::MEGU_IDLE_MID_L.render_to_bgmap(BG, (8, 0));
            let mid_r = assets::MEGU_IDLE_MID_R.render_to_bgmap(BG, (8, 10));
            let front = assets::MEGU_IDLE_FRONT.render_to_bgmap(BG, (16, 0));
            Pose {
                back_l,
                back_r,
                mid_l,
                mid_r,
                front_l: front,
                front_r: front,
            }
        };
        let celebrating_1 = {
            let back_l = assets::MEGU_CELEBRATE_1_BACK_L.render_to_bgmap(BG, (0, 20));
            let back_r = assets::MEGU_CELEBRATE_1_BACK_R.render_to_bgmap(BG, (0, 30));
            let mid_l = assets::MEGU_CELEBRATE_1_MID_L.render_to_bgmap(BG, (8, 20));
            let mid_r = assets::MEGU_CELEBRATE_1_MID_R.render_to_bgmap(BG, (8, 30));
            let front_l = assets::MEGU_CELEBRATE_1_FRONT_L.render_to_bgmap(BG, (16, 20));
            let front_r = assets::MEGU_CELEBRATE_1_FRONT_R.render_to_bgmap(BG, (16, 30));
            Pose {
                back_l,
                back_r,
                mid_l,
                mid_r,
                front_l,
                front_r,
            }
        };
        let celebrating_2 = {
            let back_l = assets::MEGU_CELEBRATE_2_BACK_L.render_to_bgmap(BG, (0, 40));
            let back_r = assets::MEGU_CELEBRATE_2_BACK_R.render_to_bgmap(BG, (0, 50));
            let mid_l = assets::MEGU_CELEBRATE_2_MID_L.render_to_bgmap(BG, (8, 40));
            let mid_r = assets::MEGU_CELEBRATE_2_MID_R.render_to_bgmap(BG, (8, 50));
            let front_l = assets::MEGU_CELEBRATE_2_FRONT_L.render_to_bgmap(BG, (16, 40));
            let front_r = assets::MEGU_CELEBRATE_2_FRONT_R.render_to_bgmap(BG, (16, 50));
            Pose {
                back_l,
                back_r,
                mid_l,
                mid_r,
                front_l,
                front_r,
            }
        };

        Self {
            idle,
            celebrating_1,
            celebrating_2,
            pos: (0, 0),
            face_right: false,
            jiggle: false,
            celebrate: None,
            counter: 0,
        }
    }

    pub fn init(&mut self, center: (i16, i16)) {
        self.pos = (center.0 - 32, center.1 - 40);
        self.face_right = false;
        self.jiggle = false;
        self.celebrate = None;
        self.counter = 0;
    }

    pub fn draw(&self, next_world: usize) -> usize {
        let pose = match self.celebrate {
            Some(false) => &self.celebrating_1,
            Some(true) => &self.celebrating_2,
            None => &self.idle,
        };
        pose.draw(self.pos, self.face_right, self.jiggle, next_world)
    }

    pub fn update(&mut self, state: &mut GameState) {
        self.counter += 1;
        if self.counter % 4 == 0 {
            let rand = state.rand().random_range(0..16);
            self.jiggle = !self.jiggle && rand == 0;
            if self.counter == 16 {
                self.face_right = rand < 4;
                self.celebrate = self.celebrate.map(|_| rand % 2 == 0);
                self.counter = 0;
            }
        }
    }

    pub fn win(&mut self) {
        self.counter = 0;
        self.jiggle = false;
        self.celebrate = Some(true);
    }
}
