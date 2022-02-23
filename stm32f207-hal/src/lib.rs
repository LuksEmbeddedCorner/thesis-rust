#![no_std]

pub mod delay;
pub mod gpio;
pub mod interrupt_free_cell;
pub mod prelude;
pub mod rcc;
pub mod time;
pub mod timer;
#[cfg(feature = "ethernet")]
pub mod ethernet;

mod sealed {
    pub trait Sealed {}
}
