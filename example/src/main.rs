#![no_std]
#![no_main]
#![allow(static_mut_refs)]
extern crate alloc;

pub mod heap;

use dogs164_rs::ssd18030_i2c::*;
use fugit::{ExtU32, RateExtU32};
use heapless::{String, format};

use embedded_hal::digital::OutputPin;
use heap::Heap;
use panic_halt as panic;
use rp_pico as bsp;

use bsp::{
    entry,
    hal::{
        Timer,
        clocks::{Clock, init_clocks_and_plls},
        gpio::FunctionI2C,
        gpio::bank0::{Gpio20, Gpio21},
        gpio::{FunctionI2c, Pin, PullNone},
        pac,
        sio::Sio,
        uart::*,
        watchdog::Watchdog,
    },
};
use defmt::export::char;
use defmt::timestamp;
use dogs164_rs::commands::{
    CMD_CLEAR_DISPLAY, CMD_RETURN_HOME, DisplaySettings, DoubleHeight, LineDisplayMode, ViewMode,
};
use dogs164_rs::config::Config;
use embedded_hal::i2c::I2c;
use rp_pico::hal::I2C;
use rp_pico::hal::gpio::bank0::{Gpio26, Gpio27};
use rp_pico::hal::gpio::{PullDown, PullUp};
use rp_pico::hal::timer::Alarm;
use rp_pico::pac::{I2C0, I2C1};

pub type I2C1Type = I2C<
    I2C1,
    (
        Pin<Gpio26, FunctionI2c, PullNone>,
        Pin<Gpio27, FunctionI2c, PullNone>,
    ),
>;

#[entry]
fn main() -> ! {
    Heap::init();
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut alarm0 = timer.alarm_0().unwrap();

    let sda: Pin<Gpio26, FunctionI2c, PullNone> = pins.gpio26.into_pull_type().into_function();
    let scl: Pin<Gpio27, FunctionI2c, PullNone> = pins.gpio27.into_pull_type().into_function();

    let i2c: I2C1Type = I2C::i2c1_with_external_pull_up(
        pac.I2C1,
        sda,
        scl,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    let mut lcd = SSD18030::new_i2c(i2c, 0x3D, &mut timer);

    let config = Config::default();

    let _ = lcd.init(config);

    let o_symbol = [
        0b11100, // top line
        0b10100, // second
        0b11100, // third
        0b00000, // fourth
        0b00000, // fifth
        0b00000, // sixth
        0b00000, // seventh
        0b00000, // bottom line
    ];
    let _ = lcd.create_custom_char(0x0, &o_symbol);

    let _ = lcd.locate(1, 1);
    alarm0.schedule(2.secs()).unwrap();
    while !alarm0.finished() {}

    let _ = lcd.write("C1: 4.12 ");
    let _ = lcd.write_special_char(0x00);
    let _ = lcd.write("C");
    alarm0.schedule(2.secs()).unwrap();
    while !alarm0.finished() {}

    let _ = lcd.locate(2, 1);
    let _ = lcd.write("C2: 3.93V");
    alarm0.schedule(2.secs()).unwrap();
    while !alarm0.finished() {}
    let _ = lcd.locate(3, 1);
    let _ = lcd.write("C3: 3.53V");

    alarm0.schedule(2.secs()).unwrap();
    while !alarm0.finished() {}

    let _ = lcd.locate(2, 1);

    let mut led_pin = pins.led.into_push_pull_output();
    led_pin.set_high().unwrap();
    loop {}
}
