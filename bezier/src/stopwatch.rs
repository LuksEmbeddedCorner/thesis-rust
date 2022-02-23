use cortex_m::peripheral::DWT;

pub struct StopWatch {
    start_value: u32,
}

impl StopWatch {
    pub fn init(dwt: &mut DWT) {
        unsafe {
            // Enable CYCNT
            dwt.ctrl.modify(|val| val | 1)
        }
    }

    pub fn start(dwt: &DWT) -> StopWatch {
        let start_value = dwt.cyccnt.read();

        Self { start_value }
    }

    pub fn stop(&self, dwt: &DWT) -> u32 {
        let stop_value = dwt.cyccnt.read();

        stop_value.wrapping_sub(self.start_value)
    }
}
