//! TODO
#![expect(clippy::cast_precision_loss, reason = "Framerate should never be that high")]

use std::time::{Duration, Instant};

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{
    OutputSettings, SimulatorDisplay, SimulatorEvent, Window,
    sdl2::{Keycode, MouseButton},
};
use kerfur_display::{KerfurDisplay, KerfurEmote};

/// The target framerate of the simulator window
const FRAMERATE: u32 = 165;
const FRAMETIME: f32 = 1.0 / FRAMERATE as f32;

fn main() {
    let mut window = Window::new("Kerfur Simulator", &OutputSettings::default());
    window.set_max_fps(FRAMERATE);

    let display = SimulatorDisplay::<Rgb888>::new(Size::new(480, 480));
    let mut kerfur = KerfurDisplay::blue(display, KerfurEmote::Neutral);

    let mut locked = false;
    let mut instant = Instant::now();

    loop {
        // Draw the kerfur display
        kerfur.clear(Rgb888::BLACK).unwrap();
        kerfur.draw(10.).unwrap();

        // Update the window and handle events
        window.update(&kerfur);
        for event in window.events() {
            match event {
                // Exit when the window is closed
                SimulatorEvent::Quit => return,
                // Toggle lock on SPACE key, preventing expression changes
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
                // Display various expressions based on input
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

        // Get elapsed time and reset the instant
        let elapsed = instant.elapsed();
        instant = Instant::now();
        // Sleep until the next frame
        std::thread::sleep(Duration::from_secs_f32(FRAMETIME).saturating_sub(elapsed));
    }
}
