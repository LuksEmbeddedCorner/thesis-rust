use core::marker::PhantomData;

use super::{Receive, Transmit};

#[repr(transparent)]
pub(crate) struct Status<T>(u32, PhantomData<T>);

impl<T> Clone for Status<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for Status<T> {}

// Common bits between the receive and the transmit status register variants
impl<T> Status<T> {
    const OWN_MASK: u32 = 1 << 31;

    pub(crate) fn is_owned(&self) -> bool {
        self.0 & Self::OWN_MASK != 0
    }

    pub(crate) fn set_owned(&mut self) {
        self.0 |= Self::OWN_MASK
    }

    const ERROR_SUMMARY_MASK: u32 = 1 << 15;

    pub(crate) fn has_error(&self) -> bool {
        self.0 & Self::ERROR_SUMMARY_MASK != 0
    }
}

// Bits, die nur in Transmit-Modus diese Bedeutung haben
impl Status<Transmit> {
    const INTERRUPT_COMPLETION_MASK: u32 = 1 << 30;
    const LAST_SEGMENT_MASK: u32 = 1 << 29;
    const FIRST_SEGMENT_MASK: u32 = 1 << 28;
    const SECOND_ADDRESS_CHAINED_MASK: u32 = 1 << 20;

    pub(crate) const fn new_transmit() -> Self {
        const DEFAULT_STATUS: u32 = Status::<Transmit>::INTERRUPT_COMPLETION_MASK
            | Status::<Transmit>::LAST_SEGMENT_MASK
            | Status::<Transmit>::FIRST_SEGMENT_MASK
            | Status::<Transmit>::SECOND_ADDRESS_CHAINED_MASK;

        Self(DEFAULT_STATUS, PhantomData)
    }

    const TRANSMIT_END_OF_RING_MASK: u32 = 1 << 21;

    pub(crate) fn set_transmit_end_of_ring(&mut self) {
        self.0 |= Self::TRANSMIT_END_OF_RING_MASK
    }
}

impl Status<Receive> {
    pub(crate) const fn new_receive() -> Self {
        const DEFAULT_STATUS: u32 = 0;

        Self(DEFAULT_STATUS, PhantomData)
    }

    const FIRST_SEGMENT_MASK: u32 = 1 << 9;
    const LAST_SEGMENT_MASK: u32 = 1 << 8;

    const FRAME_LENGTH_SHIFT: u32 = 16;
    const FRAME_LENGTH_MASK: u32 = 0x1FFF << Status::<Receive>::FRAME_LENGTH_SHIFT;

    pub(crate) unsafe fn get_frame_len(&self) -> Option<usize> {
        if self.is_start_of_frame() && self.is_end_of_frame() {
            let length = (self.0 & Status::<Receive>::FRAME_LENGTH_MASK)
                >> Status::<Receive>::FRAME_LENGTH_SHIFT;

            return Some(length as usize);
        }

        None
    }

    fn is_start_of_frame(&self) -> bool {
        self.0 & Status::<Receive>::FIRST_SEGMENT_MASK != 0
    }

    fn is_end_of_frame(&self) -> bool {
        self.0 & Status::<Receive>::LAST_SEGMENT_MASK != 0
    }
}
