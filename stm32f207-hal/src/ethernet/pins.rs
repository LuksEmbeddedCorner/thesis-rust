use crate::gpio::{output_modes::PushPull, Alternate, Pin};

use crate::sealed::Sealed;

pub struct MiiPins {
    pub transmit_clk: Pin<Alternate<PushPull, 11>, 'C', 3>,
    pub receive_clk: Pin<Alternate<PushPull, 11>, 'A', 1>,
    pub transmit_en: Pin<Alternate<PushPull, 11>, 'G', 11>,
    pub transmit_d0: Pin<Alternate<PushPull, 11>, 'G', 13>,
    pub transmit_d1: Pin<Alternate<PushPull, 11>, 'G', 14>,
    pub transmit_d2: Pin<Alternate<PushPull, 11>, 'C', 2>,
    pub transmit_d3: Pin<Alternate<PushPull, 11>, 'B', 8>,
    pub crs: Pin<Alternate<PushPull, 11>, 'H', 2>,
    pub col: Pin<Alternate<PushPull, 11>, 'H', 3>,
    pub receive_d0: Pin<Alternate<PushPull, 11>, 'C', 4>,
    pub receive_d1: Pin<Alternate<PushPull, 11>, 'C', 5>,
    pub receive_d2: Pin<Alternate<PushPull, 11>, 'H', 6>,
    pub receive_d3: Pin<Alternate<PushPull, 11>, 'H', 7>,
    pub receive_dv: Pin<Alternate<PushPull, 11>, 'A', 7>,
    pub receive_er: Pin<Alternate<PushPull, 11>, 'I', 10>,
}

pub trait EthernetPins: Sealed {}

impl Sealed for MiiPins {}
impl EthernetPins for MiiPins {}
