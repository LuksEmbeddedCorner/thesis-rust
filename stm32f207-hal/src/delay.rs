use crate::rcc::{Clocks, HSI};
use cortex_m::asm;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

pub struct Delay {
    clocks: Clocks,
    syst: SYST,
}

impl Delay {
    pub fn new(mut syst: SYST, clocks: Clocks) -> Self {
        syst.set_clock_source(SystClkSource::Core);

        Delay { syst, clocks }
    }

    pub fn release(self) -> SYST {
        self.syst
    }
}

impl DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1000)
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32)
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32)
    }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        const MAX_RVR: u32 = 0x00_FF_FF_FF;

        let mut remaining_rvr = us * (self.clocks.sysclk().0 / (HSI / 8));

        while remaining_rvr != 0 {
            let current_rvr = Ord::min(remaining_rvr, MAX_RVR);

            self.syst.set_reload(remaining_rvr);
            self.syst.clear_current();
            self.syst.enable_counter();

            // reduce remaining reset value while waiting
            remaining_rvr -= current_rvr;

            while !self.syst.has_wrapped() {
                asm::nop();
            }

            self.syst.disable_counter();
        }
    }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(us as u32)
    }
}

impl DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(us as u32)
    }
}
