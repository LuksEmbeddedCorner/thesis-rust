use cortex_m::asm;
use stm32f2::stm32f217::{rcc, RCC};

use crate::time::Hertz;

pub trait RccExtension {
    fn constrain(self) -> Rcc;
}

impl RccExtension for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            ahb1: AHB1 { _0: () },
            ahb2: AHB2 { _0: () },
            ahb3: AHB3 { _0: () },
            apb1: APB1 { _0: () },
            apb2: APB2 { _0: () },
            cfgr: CFGR {
                hclk: None,
                pclk1: None,
                pclk2: None,
                sysclk: None,
            },
        }
    }
}

pub struct Rcc {
    pub ahb1: AHB1,
    pub ahb2: AHB2,
    pub ahb3: AHB3,
    pub apb1: APB1,
    pub apb2: APB2,
    pub cfgr: CFGR,
}

pub struct AHB1 {
    _0: (),
}

impl AHB1 {
    pub fn enr(&mut self) -> &rcc::AHB1ENR {
        unsafe { &(*RCC::ptr()).ahb1enr }
    }

    pub fn rstr(&mut self) -> &rcc::AHB1RSTR {
        unsafe { &(*RCC::ptr()).ahb1rstr }
    }
}

pub struct AHB2 {
    _0: (),
}

impl AHB2 {
    pub fn enr(&mut self) -> &rcc::AHB2ENR {
        unsafe { &(*RCC::ptr()).ahb2enr }
    }

    pub fn rstr(&mut self) -> &rcc::AHB2RSTR {
        unsafe { &(*RCC::ptr()).ahb2rstr }
    }
}

pub struct AHB3 {
    _0: (),
}

impl AHB3 {
    pub fn enr(&mut self) -> &rcc::AHB3ENR {
        unsafe { &(*RCC::ptr()).ahb3enr }
    }

    pub fn rstr(&mut self) -> &rcc::AHB3RSTR {
        unsafe { &(*RCC::ptr()).ahb3rstr }
    }
}

pub struct APB1 {
    _0: (),
}

impl APB1 {
    pub fn enr(&mut self) -> &rcc::APB1ENR {
        unsafe { &(*RCC::ptr()).apb1enr }
    }

    pub fn rstr(&mut self) -> &rcc::APB1RSTR {
        unsafe { &(*RCC::ptr()).apb1rstr }
    }
}

pub struct APB2 {
    _0: (),
}

impl APB2 {
    pub fn enr(&mut self) -> &rcc::APB2ENR {
        unsafe { &(*RCC::ptr()).apb2enr }
    }

    pub fn rstr(&mut self) -> &rcc::APB2RSTR {
        unsafe { &(*RCC::ptr()).apb2rstr }
    }
}

pub struct CFGR {
    hclk: Option<u32>,
    pclk1: Option<u32>,
    pclk2: Option<u32>,
    sysclk: Option<u32>,
}

pub const HSI: u32 = 8_000_000;

impl CFGR {
    pub fn hclk(mut self, freq: impl Into<Hertz>) -> Self {
        self.hclk = Some(freq.into().0);
        self
    }

    pub fn pclk1(mut self, freq: impl Into<Hertz>) -> Self {
        self.pclk1 = Some(freq.into().0);
        self
    }

    pub fn pclk2(mut self, freq: impl Into<Hertz>) -> Self {
        self.pclk2 = Some(freq.into().0);
        self
    }

    pub fn sysclk(mut self, freq: impl Into<Hertz>) -> Self {
        self.sysclk = Some(freq.into().0);
        self
    }

    pub fn freeze(self) -> Clocks {
        const SYSCLK_MAX_FREQ: u32 = 168_000_000; // 168 Mhz
        const AHB_MAX_FREQ: u32 = 120_000_000; // 120 Mhz
        const APB2_MAX_FREQ: u32 = 60_000_000; // 60 Mhz
        const APB1_MAX_FREQ: u32 = 30_000_000; // 30 Mhz

        let pllmul = (2 * self.sysclk.unwrap_or(HSI)) / HSI;
        let pllmul = pllmul.clamp(2, 16);
        let pllmul_bits = if pllmul == 2 {
            None
        } else {
            Some(pllmul as u16 / 2)
        };

        let sysclk = pllmul * HSI / 2;
        assert!(sysclk <= SYSCLK_MAX_FREQ);

        // find best matching prescaler
        let (hpre, hpre_bits) = self
            .hclk
            .map(|hclk| match sysclk / hclk {
                0 => unreachable!(),
                1 => (1, 0b0000),
                2 => (2, 0b1000),
                3..=5 => (4, 0b1001),
                6..=11 => (8, 0b1010),
                12..=39 => (16, 0b1011),
                40..=95 => (64, 0b1100),
                96..=163 => (128, 0b1101),
                164..=383 => (256, 0b1110),
                _ => (512, 0b0111),
            })
            .unwrap_or((1, 0b1000));

        let hclk = sysclk / hpre;
        assert!(hclk <= AHB_MAX_FREQ);

        // find best matching pclk1
        let (ppre1, ppre1_bits) = self
            .pclk1
            .map(|pclk1| match hclk / pclk1 {
                0 => unreachable!(),
                1 => (1, 0b000),
                2 => (2, 0b100),
                3..=5 => (4, 0b101),
                6..=11 => (8, 0b110),
                _ => (16, 0b111),
            })
            .unwrap_or((4, 0b101));

        let pclk1 = hclk / (ppre1 as u32);
        assert!(pclk1 <= APB1_MAX_FREQ);

        // find best matching pclk2
        let (ppre2, ppre2_bits) = self
            .pclk2
            .map(|pclk2| match hclk / pclk2 {
                0 => unreachable!(),
                1 => (1, 0b000),
                2 => (2, 0b100),
                3..=5 => (4, 0b101),
                6..=11 => (8, 0b110),
                _ => (16, 0b111),
            })
            .unwrap_or((2, 0b100));

        let pclk2 = hclk / (ppre2 as u32);
        assert!(pclk2 <= APB2_MAX_FREQ);

        let rcc = unsafe { &*RCC::ptr() };
        if let Some(pllmul_bits) = pllmul_bits {
            // use PLL as source
            rcc.pllcfgr.write(|w| unsafe { w.plln().bits(pllmul_bits) });

            rcc.cr.write(|w| w.pllon().set_bit());

            while rcc.cr.read().pllrdy().bit_is_clear() {
                asm::nop();
            }

            // set cfgr
            rcc.cfgr.modify(|_, w| unsafe {
                w.ppre1()
                    .bits(ppre1_bits)
                    .ppre2()
                    .bits(ppre2_bits)
                    .hpre()
                    .bits(hpre_bits)
                    .sw()
                    .pll()
            })
        } else {
            // use HSI as source
            // set cfgr
            rcc.cfgr.modify(|_, w| unsafe {
                w.ppre1()
                    .bits(ppre1_bits)
                    .ppre2()
                    .bits(ppre2_bits)
                    .hpre()
                    .bits(hpre_bits)
                    .sw()
                    .hsi()
            })
        }

        Clocks {
            hclk: Hertz(hclk),
            pclk1: Hertz(pclk1),
            pclk2: Hertz(pclk2),
            ppre1,
            ppre2,
            sysclk: Hertz(sysclk),
        }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
pub struct Clocks {
    hclk: Hertz,
    pclk1: Hertz,
    pclk2: Hertz,
    ppre1: u8,
    ppre2: u8,
    sysclk: Hertz,
}

impl Clocks {
    /// Returns the frequency of the AHB1
    pub fn hclk(&self) -> Hertz {
        self.hclk
    }

    /// Returns the frequency of the APB1
    pub fn pclk1(&self) -> Hertz {
        self.pclk1
    }

    /// Returns the frequency of the APB2
    pub fn pclk2(&self) -> Hertz {
        self.pclk2
    }

    /// Returns the prescaler of the APB1
    pub fn ppre1(&self) -> u8 {
        self.ppre1
    }

    /// Returns the prescaler of the APB2
    pub fn ppre2(&self) -> u8 {
        self.ppre2
    }

    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
