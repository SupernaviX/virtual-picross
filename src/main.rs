#![no_main]
#![no_std]

mod assets;
mod game;
mod puzzle;
mod state;

use vb_graphics as gfx;

use crate::{
    game::Game,
    puzzle::{BOMBERMAN_BLOCK, GOLF_BALL},
    state::GameState,
};

vb_rt::rom_header!("Virtual Picross", "SG", "VPIC");
vb_rt::main!({ main() });

static FRAME: gfx::FrameMonitor = gfx::FrameMonitor::new();
vb_rt::vip_interrupt_handler!({
    FRAME.acknowledge_interrupts();
});

fn main() {
    gfx::init_display();
    gfx::set_colors(32, 64, 32);
    gfx::set_bkcol(0);
    gfx::load_character_data(&assets::ALL, 0);

    let mut game = Game::new();
    game.load_puzzle(&GOLF_BALL);
    game.load_puzzle(&BOMBERMAN_BLOCK);
    let mut state = GameState::new();

    FRAME.enable_interrupts();

    loop {
        game.draw();

        state.update();
        game.update(&state);

        FRAME.wait_for_new_frame();
    }
}
