//! adapted from stm32-eth
//! https://github.com/stm32-rs/stm32-eth

mod descriptor;
pub mod device;
pub mod pins;
mod receive;
pub mod ring;
mod transmit;

const MAX_TRANSMISSION_UNIT: usize = 1522; // VLAN Frame max size

mod sealed {
    pub trait Sealed {}
}
