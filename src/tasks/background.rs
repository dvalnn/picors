use cyw43::NetDriver;
use cyw43_pio::PioSpi;
use embassy_net::Stack;
use embassy_rp::{
    gpio::Output,
    peripherals::{DMA_CH0, PIO0},
};

#[embassy_executor::task]
pub async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
pub async fn net_task(stack: &'static Stack<NetDriver<'static>>) -> ! {
    stack.run().await
}
