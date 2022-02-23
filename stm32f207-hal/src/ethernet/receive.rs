use stm32f2::stm32f217::ETHERNET_DMA;

use super::ring::{Receive, RingEntry};

pub enum ReceiveError {
    BufferEmpty,
    FrameTruncated,
    DMAError,
}

impl RingEntry<Receive> {
    fn receive_frame(&mut self) -> Result<ReceiveFrame, ReceiveError> {
        if self.is_owned() {
            return Err(ReceiveError::BufferEmpty);
        }

        if unsafe { self.has_error() } {
            unsafe { self.set_owned() };
            return Err(ReceiveError::DMAError);
        }

        // Check if frame fit into the buffer
        // If not, set self.is_owned()
        let length = match unsafe { self.get_buffer1_len() } {
            Some(x) => x,
            // Frame did not fit into the buffer
            None => {
                unsafe { self.set_owned() };
                return Err(ReceiveError::FrameTruncated);
            }
        };

        Ok(ReceiveFrame {
            entry: self,
            length,
        })
    }
}

pub struct ReceiveFrame<'a> {
    entry: &'a mut RingEntry<Receive>,
    length: usize,
}

impl<'a> ReceiveFrame<'a> {
    pub(crate) fn as_bytes_mut(&mut self) -> &mut [u8] {
        // Safe, because a TransmitFrame only exists when:
        // - It has been checked that the entry is not currently owned by the DMA Controller
        // - As the frame holds a unique engine to the RingEntry, the buffer cannot be used elsewhere
        let bytes = unsafe { self.entry.bytes_unchecked_mut() };

        &mut bytes[..self.length]
    }
}

impl<'a> Drop for ReceiveFrame<'a> {
    fn drop(&mut self) {
        unsafe { self.entry.set_owned() }
    }
}

pub struct ReceiveRing<'a> {
    entries: &'a mut [RingEntry<Receive>],
    // Index of the entry where the next frame should be placed
    next_entry: usize,
}

impl<'a> ReceiveRing<'a> {
    pub fn new(entries: &'a mut [RingEntry<Receive>], ethernet_dma: &ETHERNET_DMA) -> Self {
        assert!(!entries.is_empty());

        let mut result = Self {
            entries,
            next_entry: 0,
        };

        result.init_ring_entry_buffers();
        result.chain_entry_descriptors();

        // Mark the entries as owned, so that the DMA controller may use them
        for entry in result.entries.iter_mut() {
            unsafe {
                entry.set_owned();
            }
        }

        // Register the descriptors
        ethernet_dma
            .dmardlar
            .write(|w| unsafe { w.srl().bits(result.entries.as_mut_ptr() as u32) });

        // Start the transmissions
        ethernet_dma.dmaomr.modify(|_, w| w.sr().set_bit());

        result
    }

    fn init_ring_entry_buffers(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.init();
        }
    }

    fn chain_entry_descriptors(&mut self) {
        let first = self.entries[0].descriptor_ptr();
        let mut previous_entry: Option<&mut RingEntry<Receive>> = None;
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
                previous_entry.set_receive_end_of_ring();
            }
        }
    }

    pub fn receive_frame(
        &mut self,
        ethernet_dma: &ETHERNET_DMA,
    ) -> Result<ReceiveFrame, ReceiveError> {
        self.request_poll(ethernet_dma);

        let entries_len = self.entries.len();
        let frame = self.entries[self.next_entry].receive_frame()?;

        // increment next entry
        self.next_entry += 1;

        if self.next_entry >= entries_len {
            self.next_entry = 0;
        }

        Ok(frame)
    }

    pub fn request_poll(&mut self, ethernet_dma: &ETHERNET_DMA) {
        ethernet_dma.dmarpdr.write(|w| unsafe { w.rpd().bits(1) })
    }
}
