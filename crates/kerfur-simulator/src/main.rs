//! TODO
#![expect(clippy::cast_precision_loss, reason = "Framerate should never be that high")]

use std::time::{Duration, Instant};

use embedded_graphics_core::{pixelcolor::Rgb888, prelude::*};
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

    let display = SimulatorDisplay::<Rgb888>::new(Size::new(480, 480));
    let mut kerfur = KerfurDisplay::blue(display, KerfurEmote::Neutral);

    let mut neutral = true;
    let mut blink_counter = 0u32;

    let mut locked = false;
    let mut instant = Instant::now();

    loop {
        // Draw the kerfur display
        kerfur.clear(Rgb888::BLACK).unwrap();
        kerfur.draw(5.).unwrap();

        // Simulate spaces between pixels
        kerfur
            .draw_iter(ScanlineIterator::<_, 2>::new(Point::new_equal(480), Rgb888::BLACK))
            .unwrap();

        // Update the window and handle events
        window.update(&kerfur);
        for event in window.events() {
            match event {
                // Exit when the window is closed
                SimulatorEvent::Quit => return,
                // Toggle lock on SPACE key, preventing expression changes
                SimulatorEvent::KeyDown { keycode: Keycode::SPACE, .. } => {
                    if locked {
                        neutral = true;
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
                    neutral = false;
                }
                SimulatorEvent::MouseButtonDown { mouse_btn: MouseButton::Right, .. } => {
                    kerfur.set_expression(KerfurEmote::Dazed);
                    neutral = false;
                }
                SimulatorEvent::KeyDown { keycode: Keycode::UP, .. } => {
                    kerfur.set_expression(KerfurEmote::NeutralUp);
                    neutral = false;
                }
                SimulatorEvent::KeyDown { keycode: Keycode::DOWN, .. } => {
                    kerfur.set_expression(KerfurEmote::NeutralDown);
                    neutral = false;
                }
                SimulatorEvent::KeyDown { keycode: Keycode::LEFT, .. } => {
                    kerfur.set_expression(KerfurEmote::NeutralLeft);
                    neutral = false;
                }
                SimulatorEvent::KeyDown { keycode: Keycode::RIGHT, .. } => {
                    kerfur.set_expression(KerfurEmote::NeutralRight);
                    neutral = false;
                }
                SimulatorEvent::MouseButtonUp { .. }
                | SimulatorEvent::KeyUp {
                    keycode: Keycode::UP | Keycode::DOWN | Keycode::LEFT | Keycode::RIGHT,
                    ..
                } => {
                    kerfur.set_expression(KerfurEmote::Neutral);
                    neutral = true;
                }
                _ => {}
            }
        }

        // Handle blinking when using the neutral expression
        if neutral {
            blink_counter += 1;
            if blink_counter >= 230 {
                blink_counter = 0;
                kerfur.set_expression(KerfurEmote::Neutral);
            } else if blink_counter == 200 {
                kerfur.set_expression(KerfurEmote::Blink);
            }
        }

        // Get elapsed time and reset the instant
        let elapsed = instant.elapsed();
        instant = Instant::now();

        // Sleep until the next frame
        std::thread::sleep(Duration::from_secs_f32(FRAMETIME).saturating_sub(elapsed));
    }
}

struct ScanlineIterator<C: PixelColor, const N: i32> {
    size: Point,
    curr: Point,
    color: C,
}

impl<C: PixelColor, const N: i32> ScanlineIterator<C, N> {
    #[must_use]
    const fn new(size: Point, color: C) -> Self { Self { size, color, curr: Point::zero() } }
}

impl<C: PixelColor, const N: i32> Iterator for ScanlineIterator<C, N> {
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.y >= self.size.y {
            None
        } else {
            loop {
                // Draw blocks of N pixels every N pixels
                if self.curr.x / N % 2 == 1 {
                    break;
                }
                // Every N rows, skip N rows
                if self.curr.y / N % 2 == 1 {
                    break;
                }

                self.curr.x += 1;
                if self.curr.x >= self.size.x {
                    self.curr.x = 0;
                    self.curr.y += 1;
                }
            }

            let output = Pixel(self.curr, self.color);

            self.curr.x += 1;
            if self.curr.x >= self.size.x {
                self.curr.x = 0;
                self.curr.y += 1;
            }

            Some(output)
        }
    }
}
