#![no_main]
#![no_std]

mod assets;
mod game;
mod menu;
mod puzzle;
mod state;

use vb_graphics as gfx;

use crate::{game::Game, menu::Menu, state::GameState};

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
    gfx::load_character_data(&puzzle::ICON_CHARS, puzzle::ICON_CHAR_OFFSET);

    let mut state = GameState::new();

    let mut menu = Menu::new();
    let mut game = Game::new();

    let mut active = ActiveScreen::Menu;

    FRAME.enable_interrupts();

    loop {
        match active {
            ActiveScreen::Menu => menu.draw(),
            ActiveScreen::Game => game.draw(),
        };

        state.update();

        match active {
            ActiveScreen::Menu => {
                if let Some(puzzle) = menu.update(&state) {
                    game.load_puzzle(puzzle);
                    active = ActiveScreen::Game;
                }
            }
            ActiveScreen::Game => {
                if game.update(&state) {
                    active = ActiveScreen::Menu;
                }
            }
        }

        FRAME.wait_for_new_frame();
    }
}

enum ActiveScreen {
    Menu,
    Game,
}
