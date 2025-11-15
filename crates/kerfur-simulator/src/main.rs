//! TODO

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{
    OutputSettings, SimulatorDisplay, SimulatorEvent, Window,
    sdl2::{Keycode, MouseButton},
};
use kerfur_display::{KerfurDisplay, KerfurEmote};

const FRAMERATE: u32 = 120;
#[expect(clippy::cast_precision_loss, reason = "Framerate should never be that high")]
const FRAMETIME: f32 = 1.0 / FRAMERATE as f32;

fn main() {
    let mut window = Window::new("Kerfur Simulator", &OutputSettings::default());
    window.set_max_fps(FRAMERATE);

    let display = SimulatorDisplay::<Rgb888>::new(Size::new(480, 480));
    let mut kerfur = KerfurDisplay::blue(display, KerfurEmote::Neutral);

    let mut locked = false;

    loop {
        kerfur.clear(Rgb888::BLACK).unwrap();
        kerfur.draw(10.).unwrap();

        window.update(kerfur.display());
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return,
                SimulatorEvent::KeyDown { keycode: Keycode::SPACE, .. } => {
                    if locked {
                        locked = false;
                        kerfur.set_expression(KerfurEmote::Neutral);
                    } else {
                        locked = true;
                    }
                }
                // Don't change expressions if locked
                _ if locked => {}
                SimulatorEvent::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                    kerfur.set_expression(KerfurEmote::Meow);
                }
                SimulatorEvent::MouseButtonDown { mouse_btn: MouseButton::Right, .. } => {
                    kerfur.set_expression(KerfurEmote::Dazed);
                }
                SimulatorEvent::KeyDown { keycode: Keycode::LEFT, .. } => {
                    kerfur.set_expression(KerfurEmote::NeutralLeft);
                }
                SimulatorEvent::KeyDown { keycode: Keycode::RIGHT, .. } => {
                    kerfur.set_expression(KerfurEmote::NeutralRight);
                }
                SimulatorEvent::MouseButtonUp { .. }
                | SimulatorEvent::KeyUp { keycode: Keycode::LEFT | Keycode::RIGHT, .. } => {
                    kerfur.set_expression(KerfurEmote::Neutral);
                }
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_secs_f32(FRAMETIME));
    }
}
