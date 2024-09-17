//! This example test the RP Pico W on board LED.
//!
//! It does not work with the RP Pico board.

#![no_std]
#![no_main]
#![allow(unused)]

mod error;
mod tasks;

use cyw43::Control;
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
    multicore::{self, spawn_core1},
    peripherals::PIO0,
    pio::{InterruptHandler, Pio},
};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use embassy_executor::{Executor, Spawner};
use embassy_time::{Duration, Timer};

use cyw43_pio::PioSpi;
use static_cell::StaticCell;

use defmt::*;
use defmt_rtt as _;

use panic_probe as _;

use tasks::{cyw43_task, net_task};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

static CYW43_CONTROL: StaticCell<Control> = StaticCell::new();

static mut CORE1_STACK: multicore::Stack<4069> = multicore::Stack::new();
static EXECUTOR_C1: StaticCell<Executor> = StaticCell::new();
static CHANNEL: Channel<CriticalSectionRawMutex, LedState, 1> = Channel::new();

enum LedState {
    On,
    Off,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    // Initialization and setup
    let p = embassy_rp::init(Default::default());
    // let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    // let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download 43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download 43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000
    let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);

    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init_with(cyw43::State::new);
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let control = CYW43_CONTROL.init(control);

    // WIFI Scan
    {
        info!("Starting WIFI scan");
        let mut scanner = control.scan(Default::default()).await;
        while let Some(bss) = scanner.next().await {
            if let Ok(ssid_str) = core::str::from_utf8(&bss.ssid) {
                info!("scanned {} = {:x}", ssid_str, bss.bssid);
            }
        }
    }

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            EXECUTOR_C1
                .init_with(Executor::new)
                .run(|spawner| unwrap!(spawner.spawn(core1_task())))
        },
    );

    info!("Starting Core 0 Loop");
    loop {
        match CHANNEL.receive().await {
            LedState::On => {
                control.gpio_set(0, true).await;
                info!("led on!");
            }
            LedState::Off => {
                control.gpio_set(0, false).await;
                info!("led off!");
            }
        }
    }
}

#[embassy_executor::task]
async fn core1_task() {
    info!("Hello from core 1!");
    let delay = Duration::from_secs(1);
    loop {
        CHANNEL.send(LedState::On).await;
        info!("led on!");
        Timer::after(delay).await;
        CHANNEL.send(LedState::Off).await;
        info!("led off!");
        Timer::after(delay).await;
    }
}
