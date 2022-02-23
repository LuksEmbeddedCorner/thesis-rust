use stm32f2::stm32f217::ETHERNET_DMA;

use super::ring::{RingEntry, Transmit};

pub enum TransmitError {
    BufferFull,
    FrameTooLong,
}

impl RingEntry<Transmit> {
    fn prepare_frame(&mut self, length: usize) -> Result<TransmitFrame, TransmitError> {
        if self.is_owned() {
            return Err(TransmitError::BufferFull);
        }

        // Safe: Entry is not accessed by the dma engine
        unsafe {
            self.set_buffer1_len(length)?;
        }

        Ok(TransmitFrame {
            entry: self,
            length,
        })
    }
}

pub struct TransmitFrame<'a> {
    entry: &'a mut RingEntry<Transmit>,
    length: usize,
}

impl<'a> TransmitFrame<'a> {
    pub(crate) fn as_bytes_mut(&mut self) -> &mut [u8] {
        // Safe, because a TransmitFrame only exists when:
        // - It has been checked that the entry is not currently owned by the DMA Controller
        // - As the frame holds a unique engine to the RingEntry, the buffer cannot be used elsewhere
        let bytes = unsafe { self.entry.bytes_unchecked_mut() };

        &mut bytes[..self.length]
    }

    fn send(self) {
        unsafe {
            self.entry.set_owned();
        }
    }
}

pub struct TransmitRing<'a> {
    entries: &'a mut [RingEntry<Transmit>],
    // Index of the entry where the next frame should be placed
    next_entry: usize,
}

impl<'a> TransmitRing<'a> {
    pub fn new(entries: &'a mut [RingEntry<Transmit>], ethernet_dma: &ETHERNET_DMA) -> Self {
        assert!(!entries.is_empty());

        let mut result = Self {
            entries,
            next_entry: 0,
        };

        result.init_ring_entry_buffers();
        result.chain_entry_descriptors();

        // Register the descriptors
        ethernet_dma
            .dmatdlar
            .write(|w| unsafe { w.stl().bits(result.entries.as_mut_ptr() as u32) });

        // Start the transmissions
        ethernet_dma.dmaomr.modify(|_, w| w.st().set_bit());

        result
    }

    fn init_ring_entry_buffers(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.init();
        }
    }

    fn chain_entry_descriptors(&mut self) {
        let first = self.entries[0].descriptor_ptr();
        let mut previous_entry: Option<&mut RingEntry<Transmit>> = None;
        for entry in self.entries.iter_mut() {
            if let Some(previous_entry) = previous_entry {
                unsafe { previous_entry.set_next_descriptor(entry.descriptor_ptr()) }
            }
            previous_entry = Some(entry);
        }

        // Chain last descriptor
        if let Some(previous_entry) = previous_entry {
            unsafe {
                previous_entry.set_next_descriptor(first);
                previous_entry.set_transmit_end_of_ring()
            }
        }
    }

    pub fn transmit_frame<F, R>(
        &mut self,
        length: usize,
        func: F,
        ethernet_dma: &ETHERNET_DMA,
    ) -> Result<R, TransmitError>
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // Prepare the buffer
        let mut frame: TransmitFrame = self.entries[self.next_entry].prepare_frame(length)?;

        // Callback fills the buffer
        let result = func(frame.as_bytes_mut());

        frame.send();

        self.increment_next_entry();

        self.request_poll(ethernet_dma);

        Ok(result)
    }

    fn increment_next_entry(&mut self) {
        self.next_entry += 1;

        if self.next_entry >= self.entries.len() {
            self.next_entry = 0;
        }
    }

    pub fn request_poll(&mut self, ethernet_dma: &ETHERNET_DMA) {
        ethernet_dma.dmatpdr.write(|w| unsafe { w.tpd().bits(1) })
    }
}
