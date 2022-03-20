use crate::rcc::Clocks;
use cortex_m::interrupt;
use smoltcp::{
    phy::{Device, DeviceCapabilities, RxToken, TxToken},
    Error,
};
use stm32f2::stm32f217::{ETHERNET_DMA, ETHERNET_MAC, ETHERNET_PTP, RCC, SYSCFG};

use super::{
    pins::EthernetPins,
    receive::{ReceiveFrame, ReceiveRing},
    ring::{Receive, RingEntry, Transmit},
    transmit::{TransmitError, TransmitRing},
    MAX_TRANSMISSION_UNIT,
};

#[derive(Debug, PartialEq, Eq)]
pub enum EthernetDeviceError {
    WrongClocks,
}

pub struct EthernetDevice<'r, 't> {
    // We take ownership of ETHERNET_MAC so that no one else can change the config
    #[allow(dead_code)]
    ethernet_mac: ETHERNET_MAC,
    ethernet_dma: ETHERNET_DMA,
    // we don't use it directly, but the extended descriptor format could otherwise be enabled
    #[allow(dead_code)]
    ethernet_ptp: ETHERNET_PTP,
    // we can allow access from ETHERNET_MMC however, because it only reads statistics about the device
    receive_ring: ReceiveRing<'r>,
    transmit_ring: TransmitRing<'t>,
}

impl<'r, 't> EthernetDevice<'r, 't> {
    /// Creates and Initializes the EthernetDevice
    ///
    /// In addition, make sure the following jumpers are set one the board:
    /// JP5 connects pin 1 and 2 to provide the clock by the external crystal
    /// JP6 connects pin 2 and 3 to select MII
    /// JP8 is fitted so that the EthernetDevice is enabled
    /// Refer to the board manual for more details on how to set up Ethernet.
    pub fn new(
        mut ethernet_mac: ETHERNET_MAC,
        mut ethernet_dma: ETHERNET_DMA,
        ethernet_ptp: ETHERNET_PTP,
        receive_buffer: &'r mut [RingEntry<Receive>],
        transmit_buffer: &'t mut [RingEntry<Transmit>],
        clocks: Clocks,
        pins: impl EthernetPins,
    ) -> Result<Self, EthernetDeviceError> {
        Self::setup_rcc();
        Self::reset_dma(&mut ethernet_dma);
        Self::setup(&mut ethernet_mac, &mut ethernet_dma, clocks)?;

        // The Ethernetpins are not used directly,
        // they are just required to be configured this way
        // in order to connect to the physical device
        let _ = pins;

        // the rings need to access the dma,
        // they start the transmissions themselves
        // when they have been constructed
        let receive_ring = ReceiveRing::new(receive_buffer, &ethernet_dma);
        let transmit_ring = TransmitRing::new(transmit_buffer, &ethernet_dma);

        let result = Self {
            ethernet_mac,
            ethernet_dma,
            ethernet_ptp,
            receive_ring,
            transmit_ring,
        };

        Ok(result)
    }

    fn setup(
        ethernet_mac: &mut ETHERNET_MAC,
        ethernet_dma: &mut ETHERNET_DMA,
        clocks: Clocks,
    ) -> Result<(), EthernetDeviceError> {
        pub const ETHERNET_MACMIIAR_CR_HCLK_DIV_16: u8 = 0b010;
        pub const ETHERNET_MACMIIAR_CR_HCLK_DIV_26: u8 = 0b011;
        pub const ETHERNET_MACMIIAR_CR_HCLK_DIV_42: u8 = 0b000;
        pub const ETHERNET_MACMIIAR_CR_HCLK_DIV_62: u8 = 0b001;

        let clock_range = match clocks.hclk().0 {
            0..=24_999_999 => return Err(EthernetDeviceError::WrongClocks),
            25_000_000..=34_999_999 => ETHERNET_MACMIIAR_CR_HCLK_DIV_16,
            35_000_000..=59_999_999 => ETHERNET_MACMIIAR_CR_HCLK_DIV_26,
            60_000_000..=99_999_999 => ETHERNET_MACMIIAR_CR_HCLK_DIV_42,
            _ => ETHERNET_MACMIIAR_CR_HCLK_DIV_62,
        };

        // MII address (clock range)
        ethernet_mac
            .macmiiar
            .modify(|_, w| unsafe { w.cr().bits(clock_range) });

        // configuration
        ethernet_mac.maccr.modify(|_, w| {
            w.cstf()
                .set_bit()
                .fes()
                .set_bit()
                .dm()
                .set_bit()
                .apcs()
                .set_bit()
                .re()
                .set_bit()
                .te()
                .set_bit()
        });

        // frame filter
        ethernet_mac
            .macffr
            .modify(|_, w| w.ra().set_bit().pm().set_bit());

        const ETHERNET_MACFCR_PAUSE_TIME: u16 = 0x100;

        // flow control
        ethernet_mac
            .macfcr
            .modify(|_, w| unsafe { w.pt().bits(ETHERNET_MACFCR_PAUSE_TIME) });

        // bus mode
        ethernet_dma.dmabmr.modify(|_, w| unsafe {
            w.aab()
                .set_bit()
                .usp()
                .set_bit()
                .rdp()
                .bits(32)
                .fb()
                .set_bit()
                .pm()
                .bits(0b01)
                .pbl()
                .bits(32)
        });

        // operation mode
        ethernet_dma.dmaomr.modify(|_, w| {
            w.dtcefd()
                .set_bit()
                .rsf()
                .set_bit()
                .dfrf()
                .set_bit()
                .tsf()
                .set_bit()
                .fef()
                .set_bit()
                .osf()
                .set_bit()
        });

        Ok(())
    }

    fn reset_dma(ethernet_dma: &mut ETHERNET_DMA) {
        ethernet_dma.dmabmr.modify(|_, w| w.sr().set_bit());

        while ethernet_dma.dmabmr.read().sr().bit_is_set() {
            // Wait for the software reset to be cleared
            // meaning the reset is completed
        }
    }

    fn setup_rcc() {
        // will only be used to change the Ethernet
        interrupt::free(|_| {
            let rcc;
            let syscfg;

            // Will only be used to set the bits affecting the MAC
            unsafe {
                rcc = &*RCC::ptr();
                syscfg = &*SYSCFG::ptr();
            }

            rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

            // Disable ethernet controller before changing clocks
            if rcc.ahb1enr.read().ethmacen().bit_is_set() {
                rcc.ahb1enr.modify(|_, w| w.ethmacen().clear_bit());
            }

            // Enable MII
            syscfg.pmc.modify(|_, w| w.mii_rmii_sel().clear_bit());

            // Set ethernet clocks
            rcc.ahb1enr.modify(|_, w| {
                w.ethmacen()
                    .set_bit()
                    .ethmactxen()
                    .set_bit()
                    .ethmacrxen()
                    .set_bit()
            });

            // Reset MAC
            rcc.ahb1rstr.modify(|_, w| w.ethmacrst().set_bit());
            rcc.ahb1rstr.modify(|_, w| w.ethmacrst().clear_bit());
        });
    }
}

impl<'a, 'r, 't> Device<'a> for EthernetDevice<'r, 't>
where
    'r: 'a,
    't: 'a,
{
    type TxToken = TransmitToken<'a, 't>;
    type RxToken = ReceiveToken<'a>;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let Self {
            ref mut receive_ring,
            ref mut transmit_ring,
            ref ethernet_dma,
            ..
        } = self;

        // Check if there is space in the receive ring
        let receive_frame = receive_ring.receive_frame(ethernet_dma).ok()?;

        Some((
            ReceiveToken { receive_frame },
            TransmitToken {
                transmit_ring,
                ethernet_dma,
            },
        ))
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        let Self {
            ref mut transmit_ring,
            ref ethernet_dma,
            ..
        } = self;

        Some(TransmitToken {
            transmit_ring,
            ethernet_dma,
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut result = DeviceCapabilities::default();

        result.max_transmission_unit = MAX_TRANSMISSION_UNIT;
        // Cycle stealing mode: Burst of a single byte to not starve the CPU from the system Bus
        result.max_burst_size = Some(1);

        result
    }
}

pub struct TransmitToken<'a, 'b> {
    transmit_ring: &'a mut TransmitRing<'b>,
    ethernet_dma: &'a ETHERNET_DMA,
}

impl<'a, 'b> TxToken for TransmitToken<'a, 'b> {
    fn consume<R, F>(
        self,
        _timestamp: smoltcp::time::Instant,
        len: usize,
        f: F,
    ) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        match self.transmit_ring.transmit_frame(len, f, self.ethernet_dma) {
            Ok(inner_result) => inner_result,
            Err(TransmitError::BufferFull) => Err(Error::Exhausted),
            Err(TransmitError::FrameTooLong) => Err(Error::NotSupported),
        }
    }
}

pub struct ReceiveToken<'a> {
    receive_frame: ReceiveFrame<'a>,
}

impl<'a> RxToken for ReceiveToken<'a> {
    fn consume<R, F>(mut self, _timestamp: smoltcp::time::Instant, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        f(self.receive_frame.as_bytes_mut())
    }
}
