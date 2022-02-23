use super::*;
use core::marker::PhantomData;

pub struct PartiallyErasedPin<MODE: PinMode, const PORT: char> {
    index: u8,
    _marker: PhantomData<MODE>,
}

impl<MODE: PinMode, const PORT: char, const INDEX: u8> From<Pin<MODE, PORT, INDEX>>
    for PartiallyErasedPin<MODE, PORT>
{
    fn from(_val: Pin<MODE, PORT, INDEX>) -> Self {
        PartiallyErasedPin {
            index: INDEX,
            _marker: PhantomData,
        }
    }
}

impl<Mode, const PORT: char> InputPin for PartiallyErasedPin<Input<Mode>, PORT>
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
        Ok(bits & (1 << self.index) == 0)
    }
}

impl<Mode, const PORT: char> OutputPin for PartiallyErasedPin<Output<Mode>, PORT>
where
    Mode: OutputMode,
{
    type Error = core::convert::Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            // write is aromic, so interrupts don't have to be disabled
            gpio_write!(PORT, bsrr, |w| w.bits(1 << self.index));
        }

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            // write is aromic, so interrupts don't have to be disabled
            gpio_write!(PORT, bsrr, |w| w.bits(1 << (self.index + 16)));
        }

        Ok(())
    }
}
