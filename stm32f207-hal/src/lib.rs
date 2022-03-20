#![no_std]

pub mod delay;
#[cfg(feature = "ethernet")]
pub mod ethernet;
pub mod gpio;
pub mod interrupt_free_cell;
pub mod prelude;
pub mod rcc;
pub mod time;
pub mod timer;

mod sealed {
    pub trait Sealed {}
}
