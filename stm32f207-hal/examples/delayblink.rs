#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;

// Needed for linking
#[allow(unused_imports)]
use stm32f2::stm32f217 as _;

use stm32f2::stm32f217::Peripherals;
use stm32f207_hal::prelude::*;

use embedded_hal::digital::v2::OutputPin;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let core_peripherals = cortex_m::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();

    let clocks = rcc.cfgr.sysclk(32.mhz()).freeze();

    let mut delay = stm32f207_hal::delay::Delay::new(core_peripherals.SYST, clocks);

    let mut led_pin = peripherals.GPIOG.split().pg6.into_push_pull_output();

    const BLINK_DELAY_MS: u32 = 500;

    loop {
        delay.delay_ms(BLINK_DELAY_MS);
        led_pin.set_high().unwrap();

        delay.delay_ms(BLINK_DELAY_MS);
        led_pin.set_low().unwrap();
    }
}
