use crate::{
    rcc::{Clocks, APB1, APB2},
    time::Hertz,
};
use embedded_hal::timer::{CountDown, Periodic};
use nb;
use stm32f2::stm32f217::{
    TIM10, TIM11, TIM12, TIM13, TIM14, TIM2, TIM3, TIM4, TIM5, TIM6, TIM7, TIM9,
};
use void::Void;

// Hardware timers
pub struct Timer<TIM> {
    clocks: Clocks,
    tim: TIM,
}

/// Interrupt events
pub enum Event {
    /// Timer timed out / count down ended
    TimeOut,
}

macro_rules! timers {
    ($($TIMX:ident, $timX:ident, $timXen:ident, $timXrst:ident, $APBX:ident, $apbX:ident;)+) => {
        $(
            impl Periodic for Timer<$TIMX> {}

            impl CountDown for Timer<$TIMX> {
                type Time = Hertz;

                // NOTE(allow) `w.psc().bits()` is safe for some timers but not for others because of
                // some SVD omission
                #[allow(unused_unsafe)]
                fn start<T>(&mut self, timeout: T)
                    where T: Into<Hertz>,
                {
                    // pause timer
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    // restart counter
                    self.tim.cnt.reset();

                    let frequency = timeout.into().0;

                    // Calculate required number of ticks
                    let ticks = self.clocks.pclk1().0 * (self.clocks.ppre1() as u32) / frequency;

                    // Setup prescaler and Auto-Reload
                    let prescaler = (ticks - 1) / (1 << 16);
                    self.tim.psc.write(|w| unsafe {w.psc().bits(prescaler as u16)});

                    let auto_reload = ticks / (prescaler + 1);
                    self.tim.arr.write(|w| unsafe { w.bits(auto_reload)});

                    // start timer
                    self.tim.cr1.modify(|_, w| w.cen().set_bit());
                }

                fn wait(&mut self) -> nb::Result<(), Void> {
                    if self.tim.sr.read().uif().bit_is_clear() {
                        Err(nb::Error::WouldBlock)
                    } else {
                        self.tim.sr.modify(|_, w| w.uif().clear_bit());
                        Ok(())
                    }
                }
            }

            impl Timer<$TIMX> {
                pub fn new(tim: $TIMX, timeout: impl Into<Hertz>, clocks: Clocks, $apbX: &mut $APBX) -> Self
                {
                    // enable and reset timer
                    $apbX.enr().modify(|_, w| w.$timXen().set_bit());
                    $apbX.rstr().modify(|_, w| w.$timXrst().set_bit());
                    $apbX.rstr().modify(|_, w| w.$timXrst().clear_bit());

                    let mut timer = Timer {
                        clocks,
                        tim,
                    };

                    timer.start(timeout);

                    timer
                }

                pub fn listen(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            // Enable update event interrupt
                            self.tim.dier.write(|w| w.uie().set_bit());
                        }
                    }
                }

                pub fn unlisten(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            // Enable update event interrupt
                            self.tim.dier.write(|w| w.uie().clear_bit());
                        }
                    }
                }

                pub fn release(self) -> $TIMX {
                    // pause timer
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    self.tim
                }
            }
        )+
    };
}

// TIM1 and TIM8 are advanced timers, and not supported
timers! {
    TIM2, tim2, tim2en, tim2rst, APB1, apb1;
    TIM3, tim3, tim3en, tim3rst, APB1, apb1;
    TIM4, tim4, tim4en, tim4rst, APB1, apb1;
    TIM5, tim5, tim5en, tim5rst, APB1, apb1;
    TIM6, tim6, tim6en, tim6rst, APB1, apb1;
    TIM7, tim7, tim7en, tim7rst, APB1, apb1;
    TIM9, tim9, tim9en, tim9rst, APB2, apb2;
    TIM10, tim10, tim10en, tim10rst, APB2, apb2;
    TIM11, tim11, tim11en, tim11rst, APB2, apb2;
    TIM12, tim12, tim12en, tim12rst, APB1, apb1;
    TIM13, tim13, tim13en, tim13rst, APB1, apb1;
    TIM14, tim14, tim14en, tim14rst, APB1, apb1;
}
