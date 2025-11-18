//! The first, "pro" processor core
//!
//! Cannot access values outside of this module.

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{gpio::AnyPin, i2s::AnyI2s, peripherals::DMA_CH0};

mod audio;

/// Spawn the task and run it on the protocol core.
pub(super) fn spawn(spawner: Spawner, peripherals: ProPeripherals<'static>) {
    defmt::info!("Spawning protocol core task...");
    spawner.must_spawn(pro(spawner, peripherals));
}

// -------------------------------------------------------------------------------------------------

/// The main task for the protocol core.
#[embassy_executor::task]
async fn pro(s: Spawner, p: ProPeripherals<'static>) -> ! {
    defmt::info!("Started protocol task!");

    // Spawn the audio task
    s.must_spawn(audio::task(audio::AudioPeripherals {
        i2s: p.i2s,
        i2s_dma: p.i2s_dma,
        i2s_sclock: p.i2s_sclock,
        i2s_mclock: p.i2s_mclock,
        i2s_lclock: p.i2s_lclock,
        i2s_dataout: p.i2s_dataout,
        i2s_soundin: p.i2s_soundin,
    }));

    loop {
        Timer::after(Duration::MAX).await;
    }
}

// -------------------------------------------------------------------------------------------------

/// The [`Peripherals`](esp_hal::Peripherals) available to the protocol core.
pub(crate) struct ProPeripherals<'a> {
    // I2S and DMA for Microphone and Speaker
    pub(super) i2s: AnyI2s<'a>,
    pub(super) i2s_dma: DMA_CH0<'a>,
    pub(super) i2s_sclock: AnyPin<'a>,
    pub(super) i2s_mclock: AnyPin<'a>,
    pub(super) i2s_lclock: AnyPin<'a>,
    pub(super) i2s_dataout: AnyPin<'a>,
    pub(super) i2s_soundin: AnyPin<'a>,
}
