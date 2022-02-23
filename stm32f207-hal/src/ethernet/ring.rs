use core::sync::atomic::{compiler_fence, Ordering};

use vcell::VolatileCell;

use super::{
    descriptor::{Descriptor, SetBufferError},
    transmit::TransmitError,
    MAX_TRANSMISSION_UNIT,
};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Receive;
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Transmit;

#[repr(C)]
pub struct RingEntry<T> {
    // An inner type wrapped in UnsafeCell is used here,
    // because the InnerRingEntry may always be aliased and accessed by the DMA Controller.
    // Therefore, the Ownership rules may be violated
    descriptor: Descriptor<T>,
    buffer: VolatileCell<[u8; MAX_TRANSMISSION_UNIT]>,
}

impl<T> RingEntry<T> {
    /// MUST be called once, after the RingEntry was moved into position
    pub(crate) fn init(&mut self) {
        assert!(!self.is_owned());

        unsafe {
            let bytes = self.bytes_unchecked_mut();
            let ptr = bytes.as_mut_ptr();
            let len = bytes.len();
            self.descriptor
                .set_buffer1(ptr, len)
                .expect("Could not set buffer");
        }
    }

    pub(crate) fn is_owned(&self) -> bool {
        self.descriptor.is_owned()
    }

    pub(crate) unsafe fn has_error(&self) -> bool {
        self.descriptor.has_error()
    }

    /// Safety: descriptor must be configured properly - init must have been called
    pub(crate) unsafe fn set_owned(&mut self) {
        self.descriptor.set_owned();
    }

    pub(crate) unsafe fn bytes_unchecked_mut(&mut self) -> &mut [u8] {
        // Unsafe cast: volatile &[u8] to [u8]
        // The compiler fences should prevent this from reordering to an earlier position

        compiler_fence(Ordering::SeqCst);

        let result = &mut *self.buffer.as_ptr();

        compiler_fence(Ordering::SeqCst);

        result
    }

    pub(crate) fn descriptor_ptr(&self) -> *const Descriptor<T> {
        &self.descriptor
    }

    pub(crate) unsafe fn set_next_descriptor(&mut self, next: *const Descriptor<T>) {
        self.descriptor.set_next_descriptor(next);
    }
}

impl RingEntry<Transmit> {
    pub const fn new_transmit() -> Self {
        Self {
            descriptor: Descriptor::new_transmit(),
            buffer: VolatileCell::new([0; MAX_TRANSMISSION_UNIT]),
        }
    }

    pub(crate) unsafe fn set_buffer1_len(&mut self, len: usize) -> Result<(), TransmitError> {
        self.descriptor
            .set_buffer1_len(len)
            .map_err(|err| match err {
                SetBufferError::TooLongBufferLen => TransmitError::FrameTooLong,
            })
    }

    pub(crate) unsafe fn set_transmit_end_of_ring(&mut self) {
        self.descriptor.set_transmit_end_of_ring()
    }
}

impl RingEntry<Receive> {
    pub const fn new_receive() -> Self {
        Self {
            descriptor: Descriptor::new_receive(),
            buffer: VolatileCell::new([0; MAX_TRANSMISSION_UNIT]),
        }
    }

    pub(crate) unsafe fn get_buffer1_len(&mut self) -> Option<usize> {
        self.descriptor.get_frame_len()
    }

    pub(crate) unsafe fn set_receive_end_of_ring(&mut self) {
        self.descriptor.set_receive_end_of_ring()
    }
}
