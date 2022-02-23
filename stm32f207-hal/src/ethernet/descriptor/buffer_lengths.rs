use core::marker::PhantomData;

use crate::ethernet::ring::{Receive, Transmit};

use super::SetBufferError;

#[repr(transparent)]
pub(crate) struct BufferLengths<T>(u32, PhantomData<T>);

impl<T> Clone for BufferLengths<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for BufferLengths<T> {}

impl<T> BufferLengths<T> {
    const BUFFER1_LEN_MASK: u32 = 0x1FFF;

    /// len has to be smaller than 0xFFF
    pub(crate) fn set_buffer1_len(&mut self, len: usize) -> Result<(), SetBufferError> {
        if len >= Self::BUFFER1_LEN_MASK as usize {
            return Err(SetBufferError::TooLongBufferLen);
        }

        let zeroed = self.0 & !Self::BUFFER1_LEN_MASK;

        self.0 = zeroed | (len as u32 & Self::BUFFER1_LEN_MASK);

        Ok(())
    }
}

impl BufferLengths<Transmit> {
    pub(crate) const fn new_transmit() -> Self {
        const DEFAULT_STATUS_MASK: u32 = 0;

        Self(DEFAULT_STATUS_MASK, PhantomData)
    }
}

impl BufferLengths<Receive> {
    const SECOND_ADDRESS_CHAINED_MASK: u32 = 1 << 14;

    pub(crate) const fn new_receive() -> Self {
        const DEFAULT_STATUS: u32 = BufferLengths::<Receive>::SECOND_ADDRESS_CHAINED_MASK;

        Self(DEFAULT_STATUS, PhantomData)
    }

    const RECEIVE_END_OF_RING_MASK: u32 = 1 << 15;

    pub(crate) fn set_receive_end_of_ring(&mut self) {
        self.0 |= Self::RECEIVE_END_OF_RING_MASK;
    }
}
