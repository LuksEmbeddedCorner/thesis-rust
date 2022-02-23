use super::*;
use paste::paste;
use stm32f2::stm32f217::RCC;
use stm32f2::stm32f217::{GPIOA, GPIOB, GPIOC, GPIOD, GPIOE, GPIOF, GPIOG, GPIOH, GPIOI};

pub trait GpioExtension {
    type Parts;

    fn split(self) -> Self::Parts;
}

macro_rules! gpio {
    ($GPIOX:ident, $GpioX:ident, $gpiox:ident, $letter:expr, {
        $($field:ident: $mode:ty, $i:expr ,)+
    }) => {
        paste! {
            pub struct [<$GpioX Pins>] {
                $(
                    pub $field: Pin<$mode, $letter, $i>,
                )+
            }

            impl GpioExtension for $GPIOX {
                type Parts = [<$GpioX Pins>];

                fn split(self) -> Self::Parts {
                    // Only the sections for the port will be used
                    let rcc = unsafe { &(*RCC::ptr()) };

                    // Enable the Port and reset it to the original configuration
                    cortex_m::interrupt::free(|_| {
                        rcc.ahb1enr.modify(|_, w| w.[<$gpiox en>]().enabled());
                        rcc.ahb1rstr.modify(|_, w| w.[<$gpiox rst>]().set_bit());
                        rcc.ahb1rstr.modify(|_, w| w.[<$gpiox rst>]().clear_bit());
                    });

                    Self::Parts {
                        $(
                            $field: Pin { _marker: PhantomData },
                        )+
                    }
                }
            }
        }
    };
    ($GPIOX:ident, $GpioX:ident, $gpiox:ident, $letter:expr, $letterIdent:ident) => {
        // All pins available, all pins floating input
        paste!{
            gpio! ($GPIOX, $GpioX, $gpiox, $letter, {
                [<p $letterIdent 0>]: Input<Floating>, 0,
                [<p $letterIdent 1>]: Input<Floating>, 1,
                [<p $letterIdent 2>]: Input<Floating>, 2,
                [<p $letterIdent 3>]: Input<Floating>, 3,
                [<p $letterIdent 4>]: Input<Floating>, 4,
                [<p $letterIdent 5>]: Input<Floating>, 5,
                [<p $letterIdent 6>]: Input<Floating>, 6,
                [<p $letterIdent 7>]: Input<Floating>, 7,
                [<p $letterIdent 8>]: Input<Floating>, 8,
                [<p $letterIdent 9>]: Input<Floating>, 9,
                [<p $letterIdent 10>]: Input<Floating>, 10,
                [<p $letterIdent 11>]: Input<Floating>, 11,
                [<p $letterIdent 12>]: Input<Floating>, 12,
                [<p $letterIdent 13>]: Input<Floating>, 13,
                [<p $letterIdent 14>]: Input<Floating>, 14,
                [<p $letterIdent 15>]: Input<Floating>, 15,
            });
        }
    }
}

gpio!(GPIOA, GpioA, gpioa, 'A', {
    pa0: Input<Floating>, 0,
    pa1: Input<Floating>, 1,
    pa2: Input<Floating>, 2,
    pa3: Input<Floating>, 3,
    pa4: Input<Floating>, 4,
    pa5: Input<Floating>, 5,
    pa6: Input<Floating>, 6,
    pa7: Input<Floating>, 7,
    pa8: Input<Floating>, 8,
    pa9: Input<Floating>, 9,
    pa10: Input<Floating>, 10,
    pa11: Input<Floating>, 11,
    pa12: Input<Floating>, 12,
    pa13: Input<PullUp>, 13,
    pa14: Input<PullDown>, 14,
    pa15: Input<PullUp>, 15,
});

gpio!(GPIOB, GpioB, gpiob, 'B', {
    pb0: Input<Floating>, 0,
    pb1: Input<Floating>, 1,
    pb2: Input<Floating>, 2,
    pb3: Input<Floating>, 3,
    pb4: Input<PullUp>, 4,
    pb5: Input<Floating>, 5,
    pb6: Input<Floating>, 6,
    pb7: Input<Floating>, 7,
    pb8: Input<Floating>, 8,
    pb9: Input<Floating>, 9,
    pb10: Input<Floating>, 10,
    pb11: Input<Floating>, 11,
    pb12: Input<Floating>, 12,
    pb13: Input<Floating>, 13,
    pb14: Input<Floating>, 14,
    pb15: Input<Floating>, 15,
});

gpio!(GPIOC, GpioC, gpioc, 'C', c);
gpio!(GPIOD, GpioD, gpiod, 'D', d);
gpio!(GPIOE, GpioE, gpioe, 'E', e);
gpio!(GPIOF, GpioF, gpiof, 'F', f);
gpio!(GPIOG, GpioG, gpiog, 'G', g);
gpio!(GPIOH, GpioH, gpioh, 'H', h);
gpio!(GPIOI, GpioI, gpioi, 'I', i);
