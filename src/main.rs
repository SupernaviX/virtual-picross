#![no_main]
#![no_std]

mod assets;
mod game;
mod menu;
mod puzzle;
mod save;
mod state;
mod title;

use vb_graphics as gfx;
use vb_rt::sys::vip::Character;

use crate::{game::Game, menu::Menu, state::GameState, title::Title};

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

    let mut loaded_chardata: &[Character] = &assets::TITLE;
    gfx::load_character_data(loaded_chardata, 0);
    gfx::load_character_data(&puzzle::ICON_CHARS, puzzle::ICON_CHAR_OFFSET);

    let mut state = GameState::new();

    let mut title = Title::new();
    let mut menu = Menu::new();
    let mut game = Game::new();

    let mut active = ActiveScreen::Title;
    let mut transition = Some(Transition::FadeIn(0));

    FRAME.enable_interrupts();

    loop {
        match active {
            ActiveScreen::Title => title.draw(),
            ActiveScreen::Menu => menu.draw(),
            ActiveScreen::Game => game.draw(),
        };

        FRAME.wait_for_new_frame();

        state.update();

        match &mut transition {
            Some(Transition::FadeIn(amount)) => {
                *amount += 1;
                gfx::set_colors(*amount, *amount * 2, *amount);
                if *amount == 32 {
                    transition = None;
                }
            }
            Some(Transition::FadeOut(amount, next)) => {
                *amount -= 1;
                gfx::set_colors(*amount, *amount * 2, *amount);
                if *amount == 0 {
                    let chardata: &[Character] = match next {
                        ActiveScreen::Title => &assets::TITLE,
                        _ => &assets::ALL,
                    };
                    if !core::ptr::addr_eq(loaded_chardata, chardata) {
                        loaded_chardata = chardata;
                        gfx::load_character_data(loaded_chardata, 0);
                    }
                    match next {
                        ActiveScreen::Game => game.init(),
                        ActiveScreen::Menu => menu.init(),
                        _ => {}
                    }
                    active = *next;
                    transition = Some(Transition::FadeIn(0));
                }
            }
            None => match active {
                ActiveScreen::Title => {
                    if title.update(&state) {
                        transition = Some(Transition::FadeOut(31, ActiveScreen::Menu));
                    }
                }
                ActiveScreen::Menu => {
                    if let Some(puzzle) = menu.update(&state) {
                        game.load_puzzle(puzzle);
                        transition = Some(Transition::FadeOut(31, ActiveScreen::Game));
                    }
                }
                ActiveScreen::Game => {
                    if let Some(result) = game.update(&mut state) {
                        menu.finish_puzzle(result);
                        transition = Some(Transition::FadeOut(31, ActiveScreen::Menu));
                    }
                }
            },
        }
    }
}

#[derive(Clone, Copy)]
enum ActiveScreen {
    Title,
    Menu,
    Game,
}

enum Transition {
    FadeOut(u8, ActiveScreen),
    FadeIn(u8),
}
