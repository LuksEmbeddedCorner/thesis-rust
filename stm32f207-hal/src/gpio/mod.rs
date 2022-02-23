pub mod gpio_extension;
mod macros;
pub mod partially_erased_pin;
use macros::*;

use core::marker::PhantomData;

use crate::sealed::Sealed;

pub trait PinMode: Sealed {}
pub trait InputMode: Sealed {}
pub trait OutputMode: Sealed {}

use embedded_hal::digital::v2::{InputPin, OutputPin};

pub struct NotConfigured;

pub struct Input<MODE: InputMode> {
    _marker: PhantomData<MODE>,
}

impl Sealed for NotConfigured {}
impl PinMode for NotConfigured {}
impl<Mode: InputMode> Sealed for Input<Mode> {}
impl<Mode: InputMode> PinMode for Input<Mode> {}

pub mod input_modes {
    pub struct Floating;
    pub struct PullUp;
    pub struct PullDown;
}
use input_modes::*;

impl Sealed for Floating {}
impl InputMode for Floating {}
impl Sealed for PullUp {}
impl InputMode for PullUp {}
impl Sealed for PullDown {}
impl InputMode for PullDown {}

pub mod output_modes {
    pub struct PushPull;
    pub struct OpenDrain;
}
use output_modes::*;

use crate::gpio::macros::gpio_modify;

impl Sealed for PushPull {}
impl OutputMode for PushPull {}
impl Sealed for OpenDrain {}
impl OutputMode for OpenDrain {}

pub struct Output<MODE: OutputMode> {
    _marker: PhantomData<MODE>,
}

// FIXME check that a < 16 when const generic bounds are stabilized
pub struct Alternate<MODE: OutputMode, const ALT_MODE: u8> {
    _marker: PhantomData<MODE>,
}

impl<Mode: OutputMode> Sealed for Output<Mode> {}
impl<Mode: OutputMode> PinMode for Output<Mode> {}
impl<Mode: OutputMode, const ALT_MODE: u8> Sealed for Alternate<Mode, ALT_MODE> {}
impl<Mode: OutputMode, const ALT_MODE: u8> PinMode for Alternate<Mode, ALT_MODE> {}

pub struct Pin<MODE: PinMode, const PORT: char, const INDEX: u8> {
    _marker: PhantomData<MODE>,
}

impl<MODE: PinMode, const PORT: char, const INDEX: u8> Pin<MODE, PORT, INDEX> {
    pub fn into_floating_input(self) -> Pin<Input<Floating>, PORT, INDEX> {
        let offset = 2 * INDEX;

        cortex_m::interrupt::free(|_| unsafe {
            gpio_modify!(PORT, pupdr, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b11 << offset);
                w.bits(bits)
            });
        });

        Pin {
            _marker: PhantomData,
        }
    }

    pub fn into_pull_up_input(self) -> Pin<Input<PullUp>, PORT, INDEX> {
        let offset = 2 * INDEX;

        cortex_m::interrupt::free(|_| unsafe {
            gpio_modify!(PORT, pupdr, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b11 << offset);
                bits |= 0b01 << offset;
                w.bits(bits)
            });
        });

        Pin {
            _marker: PhantomData,
        }
    }

    pub fn into_pull_down_input(self) -> Pin<Input<PullDown>, PORT, INDEX> {
        let offset = 2 * INDEX;

        cortex_m::interrupt::free(|_| unsafe {
            gpio_modify!(PORT, pupdr, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b11 << offset);
                bits |= 0b10 << offset;
                w.bits(bits)
            });
        });

        Pin {
            _marker: PhantomData,
        }
    }

    pub fn into_open_drain_output(self) -> Pin<Output<OpenDrain>, PORT, INDEX> {
        let offset = 2 * INDEX;

        cortex_m::interrupt::free(|_| unsafe {
            // Set to output mode
            gpio_modify!(PORT, moder, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b11 << offset);
                bits |= 0b01 << offset;
                w.bits(bits)
            });

            // Set open drain
            gpio_modify!(PORT, otyper, |r, w| {
                let mut bits = r.bits();
                bits |= 0b1 << INDEX;
                w.bits(bits)
            });

            // Set very high speed
            gpio_modify!(PORT, ospeedr, |r, w| {
                let mut bits = r.bits();
                bits |= 0b11 << offset;
                w.bits(bits)
            });

            // Disable Pullup or Pulldown
            gpio_modify!(PORT, pupdr, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b11 << offset);
                w.bits(bits)
            });
        });

        Pin {
            _marker: PhantomData,
        }
    }

    pub fn into_push_pull_output(self) -> Pin<Output<PushPull>, PORT, INDEX> {
        let offset = 2 * INDEX;

        cortex_m::interrupt::free(|_| unsafe {
            // Set to output mode
            gpio_modify!(PORT, moder, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b11 << offset);
                bits |= 0b01 << offset;
                w.bits(bits)
            });

            // Set push pull
            gpio_modify!(PORT, otyper, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b1 << INDEX);
                w.bits(bits)
            });

            // Set very high speed
            gpio_modify!(PORT, ospeedr, |r, w| {
                let mut bits = r.bits();
                bits |= 0b11 << offset;
                w.bits(bits)
            });

            // Disable Pullup or Pulldown
            gpio_modify!(PORT, pupdr, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b11 << offset);
                w.bits(bits)
            });
        });

        Pin {
            _marker: PhantomData,
        }
    }

    pub fn into_alternate<const ALT_MODE: u8>(
        self,
    ) -> Pin<Alternate<PushPull, ALT_MODE>, PORT, INDEX> {
        let offset = 2 * INDEX;

        assert!(ALT_MODE < 16);

        cortex_m::interrupt::free(|_| unsafe {
            // Set to alternate mode
            gpio_modify!(PORT, moder, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b11 << offset);
                bits |= 0b10 << offset;
                w.bits(bits)
            });

            // Set push pull
            gpio_modify!(PORT, otyper, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b1 << INDEX);
                w.bits(bits)
            });

            // Set very high speed
            gpio_modify!(PORT, ospeedr, |r, w| {
                let mut bits = r.bits();
                bits |= 0b11 << offset;
                w.bits(bits)
            });

            // Disable Pullup or Pulldown
            gpio_modify!(PORT, pupdr, |r, w| {
                let mut bits = r.bits();
                bits &= !(0b11 << offset);
                w.bits(bits)
            });

            // Select the alternate function
            if INDEX < 8 {
                let alternate_function_offset = INDEX * 4;

                gpio_modify!(PORT, afrl, |r, w| {
                    let mut bits = r.bits();
                    bits &= !(0b1111 << alternate_function_offset);
                    bits |= (ALT_MODE as u32) << alternate_function_offset;
                    w.bits(bits)
                });
            } else {
                let alternate_function_offset = (INDEX - 8) * 4;

                gpio_modify!(PORT, afrh, |r, w| {
                    let mut bits = r.bits();
                    bits &= !(0b1111 << alternate_function_offset);
                    bits |= (ALT_MODE as u32) << alternate_function_offset;
                    w.bits(bits)
                });
            }
        });

        Pin {
            _marker: PhantomData,
        }
    }
}

impl<Mode, const PORT: char, const INDEX: u8> InputPin for Pin<Input<Mode>, PORT, INDEX>
where
    Mode: InputMode,
{
    type Error = core::convert::Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let low = self.is_low()?;
        Ok(!low)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        let bits = unsafe { gpio_read_bits!(PORT, pupdr) };
        Ok(bits & (1 << INDEX) == 0)
    }
}

impl<Mode, const PORT: char, const INDEX: u8> OutputPin for Pin<Output<Mode>, PORT, INDEX>
where
    Mode: OutputMode,
{
    type Error = core::convert::Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            // write is aromic, so interrupts don't have to be disabled
            gpio_write!(PORT, bsrr, |w| w.bits(1 << (INDEX + 16)));
        }

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            // write is aromic, so interrupts don't have to be disabled
            gpio_write!(PORT, bsrr, |w| w.bits(0b1 << INDEX));
        }

        Ok(())
    }
}
