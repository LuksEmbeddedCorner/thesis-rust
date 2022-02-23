use core::ptr::{null, null_mut};

use vcell::VolatileCell;

mod status;
use status::*;

mod buffer_lengths;
use buffer_lengths::*;

use super::ring::{Receive, Transmit};

#[repr(C)]
pub struct Descriptor<T> {
    status: VolatileCell<Status<T>>,
    buffer_lengths: VolatileCell<BufferLengths<T>>,
    buffer1: VolatileCell<*mut u8>,
    // Could also be a pointer to a second buffer,
    // but we will only use the chained mode where it points to the next descriptor.
    next_descriptor: VolatileCell<*const Descriptor<T>>,
}

impl Descriptor<Transmit> {
    pub(crate) const fn new_transmit() -> Self {
        Self {
            status: VolatileCell::new(Status::new_transmit()),
            buffer_lengths: VolatileCell::new(BufferLengths::new_transmit()),
            buffer1: VolatileCell::new(null_mut()),
            next_descriptor: VolatileCell::new(null()),
        }
    }

    pub(crate) unsafe fn set_transmit_end_of_ring(&mut self) {
        let mut status = self.status.get();

        status.set_transmit_end_of_ring();

        self.status.set(status)
    }
}

impl Descriptor<Receive> {
    pub(crate) const fn new_receive() -> Self {
        Self {
            status: VolatileCell::new(Status::new_receive()),
            buffer_lengths: VolatileCell::new(BufferLengths::new_receive()),
            buffer1: VolatileCell::new(null_mut()),
            next_descriptor: VolatileCell::new(null()),
        }
    }

    /// Returns the length of the frame in buffer1.
    ///
    /// If the frame did not fit in buffer1 and was trundcated, none is returned
    pub(crate) unsafe fn get_frame_len(&self) -> Option<usize> {
        self.status.get().get_frame_len()
    }

    pub(crate) unsafe fn set_receive_end_of_ring(&mut self) {
        let mut buffer_lengths = self.buffer_lengths.get();

        buffer_lengths.set_receive_end_of_ring();

        self.buffer_lengths.set(buffer_lengths)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SetBufferError {
    TooLongBufferLen,
}

impl<T> Descriptor<T> {
    pub(crate) fn is_owned(&self) -> bool {
        self.status.get().is_owned()
    }

    /// Pass Ownership to the DMA Controller
    pub(crate) unsafe fn set_owned(&mut self) {
        let mut status = self.status.get();

        status.set_owned();

        self.status.set(status);
    }

    pub(crate) unsafe fn has_error(&self) -> bool {
        self.status.get().has_error()
    }

    pub(crate) unsafe fn set_buffer1(
        &mut self,
        buf: *mut u8,
        len: usize,
    ) -> Result<(), SetBufferError> {
        self.set_buffer1_len(len)?;
        self.buffer1.set(buf);

        Ok(())
    }

    pub(crate) unsafe fn set_buffer1_len(&mut self, len: usize) -> Result<(), SetBufferError> {
        let mut buffer_lengths = self.buffer_lengths.get();

        buffer_lengths.set_buffer1_len(len)?;

        self.buffer_lengths.set(buffer_lengths);

        Ok(())
    }

    pub(crate) unsafe fn set_next_descriptor(&mut self, next: *const Descriptor<T>) {
        self.next_descriptor.set(next);
    }
}
