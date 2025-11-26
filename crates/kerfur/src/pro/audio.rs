//! TODO

use embassy_time::Timer;
use esp_hal::{
    dma_buffers,
    i2s::{AnyI2s, master::I2s},
    peripherals::DMA_CH1,
};

/// A task that handles listening to and playing audio.
#[embassy_executor::task]
pub(super) async fn task(p: AudioPeripherals<'static>) -> ! {
    // Create DMA buffers
    let (_rx_buf, _rx_desc, _tx_buf, _tx_desc) = dma_buffers!(4 * 1024);

    // Wait until the audio peripherals are configured
    let config = crate::signal::AUDIO_CFG.wait().await;

    // Initialize I2S
    defmt::info!("Initializing I2S...");
    let _i2s = defmt::unwrap!(I2s::new(p.i2s, p.i2s_dma, config));
    // let i2s = i2s.with_mclk(p.i2s_mclock).into_async();

    // // Configure I2S RX for ES8311
    // let _i2s_rx = i2s
    //     .i2s_rx
    //     .with_bclk(p.i2s_sclock)
    //     .with_ws(p.i2s_lclock)
    //     .with_din(p.i2s_soundin)
    //     .build(rx_desc);

    // // Configure I2S TX for ES7210
    // let _i2s_tx = i2s.i2s_tx.with_dout(p.i2s_dataout).build(tx_desc);

    loop {
        Timer::after_secs(30).await;
    }
}

// -------------------------------------------------------------------------------------------------

pub(super) struct AudioPeripherals<'a> {
    // pub(super) amplifier_enable: TCA_P0<'a, NoopRawMutex, AsyncI2C<'a>>,
    pub(super) i2s: AnyI2s<'a>,
    pub(super) i2s_dma: DMA_CH1<'a>,
    // pub(super) i2s_sclock: AnyPin<'a>, // bclk
    // pub(super) i2s_mclock: AnyPin<'a>,
    // pub(super) i2s_lclock: AnyPin<'a>,  // ws
    // pub(super) i2s_dataout: AnyPin<'a>, // tx
    // pub(super) i2s_soundin: AnyPin<'a>, // rx
}
