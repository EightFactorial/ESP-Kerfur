//! TODO

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{
    OutputSettings, SimulatorDisplay, SimulatorEvent, Window, sdl2::MouseButton,
};
use kerfur_display::{KerfurDisplay, KerfurEmote};

const FRAMERATE: u32 = 60;
#[expect(clippy::cast_precision_loss, reason = "Framerate should never be that high")]
const FRAMETIME: f32 = 1.0 / FRAMERATE as f32;

fn main() {
    let mut window = Window::new("Kerfur Simulator", &OutputSettings::default());
    window.set_max_fps(FRAMERATE);

    let display = SimulatorDisplay::<Rgb888>::new(Size::new(480, 480));
    let mut kerfur = KerfurDisplay::blue(display, KerfurEmote::Neutral);

    let mut emote = KerfurEmote::Neutral;
    loop {
        window.update(kerfur.display());
        kerfur.clear(Rgb888::BLACK).unwrap();
        kerfur.draw(FRAMETIME).unwrap();

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return,
                SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
                    if matches!(mouse_btn, MouseButton::Left) {
                        match emote {
                            KerfurEmote::Neutral => {
                                emote = KerfurEmote::Meow;
                                kerfur.set_expression(KerfurEmote::Meow);
                            }
                            KerfurEmote::Meow => {
                                emote = KerfurEmote::Dazed;
                                kerfur.set_expression(KerfurEmote::Dazed);
                            }
                            KerfurEmote::Dazed => {
                                emote = KerfurEmote::Neutral;
                                kerfur.set_expression(KerfurEmote::Neutral);
                            }
                        }
                    } else {
                        println!("Mouse clicked at: {point:?}");
                    }
                }
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_secs_f32(FRAMETIME));
    }
}
