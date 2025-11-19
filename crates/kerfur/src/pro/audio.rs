//! TODO

use embassy_time::{Duration, Timer};
use esp_hal::{
    dma_buffers,
    gpio::AnyPin,
    i2s::{
        AnyI2s,
        master::{Channels, Config as I2sConfig, I2s},
    },
    peripherals::DMA_CH0,
    time::Rate,
};

/// A task that handles listening to and playing audio.
#[embassy_executor::task]
pub(super) async fn task(p: AudioPeripherals<'static>) -> ! {
    // Wait until the audio peripherals are configured
    crate::signal::AUDIO_ENABLE.wait().await;

    defmt::info!("Preparing microphone and speakers...");

    // Create DMA buffers
    let (_rx_buf, rx_desc, _tx_buf, tx_desc) = dma_buffers!(4 * 1024);

    // Configure I2S
    let config = I2sConfig::new_tdm_philips()
        .with_channels(Channels::MONO)
        .with_sample_rate(Rate::from_hz(22050));
    let i2s = defmt::unwrap!(I2s::new(p.i2s, p.i2s_dma, config));
    let i2s = i2s.with_mclk(p.i2s_mclock).into_async();

    // Configure I2S RX for ES8311
    let _i2s_rx = i2s
        .i2s_rx
        .with_bclk(p.i2s_sclock)
        .with_ws(p.i2s_lclock)
        .with_din(p.i2s_soundin)
        .build(rx_desc);

    // Configure I2S TX for ES7210
    let _i2s_tx = i2s.i2s_tx.with_dout(p.i2s_dataout).build(tx_desc);

    loop {
        Timer::after(Duration::MAX).await;
    }
}

// -------------------------------------------------------------------------------------------------

pub(super) struct AudioPeripherals<'a> {
    // pub(super) amplifier_enable: TCA_P0<'a, NoopRawMutex, AsyncI2C<'a>>,
    pub(super) i2s: AnyI2s<'a>,
    pub(super) i2s_dma: DMA_CH0<'a>,
    pub(super) i2s_sclock: AnyPin<'a>, // bclk
    pub(super) i2s_mclock: AnyPin<'a>,
    pub(super) i2s_lclock: AnyPin<'a>,  // ws
    pub(super) i2s_dataout: AnyPin<'a>, // tx
    pub(super) i2s_soundin: AnyPin<'a>, // rx
}
