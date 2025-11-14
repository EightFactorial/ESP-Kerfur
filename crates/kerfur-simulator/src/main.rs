//! TODO

use std::time::Instant;

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{
    OutputSettings, SimulatorDisplay, SimulatorEvent, Window, sdl2::MouseButton,
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

    let mut instant = Instant::now();
    let mut timer_enabled = false;

    loop {
        kerfur.clear(Rgb888::BLACK).unwrap();
        kerfur.draw(10.).unwrap();

        window.update(kerfur.display());
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return,
                SimulatorEvent::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                    timer_enabled = true;
                    instant = Instant::now();
                    kerfur.set_expression(KerfurEmote::Meow);
                }
                SimulatorEvent::MouseButtonDown { mouse_btn: MouseButton::Right, .. } => {
                    timer_enabled = true;
                    instant = Instant::now();
                    kerfur.set_expression(KerfurEmote::Dazed);
                }
                _ => {}
            }
        }

        if timer_enabled && instant.elapsed().as_secs_f32() >= 0.5 {
            kerfur.set_expression(KerfurEmote::Neutral);
            timer_enabled = false;
        }

        std::thread::sleep(std::time::Duration::from_secs_f32(FRAMETIME));
    }
}
