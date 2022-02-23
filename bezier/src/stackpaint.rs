extern "C" {
    static mut _stack_start: u32;
}

pub struct StackMeasurer {
    stack_end: *const u32,
}

const STACK_PATTERN: u32 = 0xABABABAB;
const PAINT_SIZE: isize = 500;

pub fn paint() -> StackMeasurer {
    unsafe {
        let mut current_stack: *mut u32;
        asm!("MOV {}, SP", out(reg_thumb) current_stack);
        let stack_end = current_stack.offset(-PAINT_SIZE);

        // Add a bit fo buffer
        current_stack = current_stack.offset(-0xf);

        while current_stack as *const _ != stack_end {
            current_stack.write_volatile(STACK_PATTERN);
            current_stack = current_stack.offset(-1);
        }

        StackMeasurer { stack_end }
    }
}

impl StackMeasurer {
    pub fn get(&self) -> usize {
        unsafe {
            let stack_end = self.stack_end as *mut u32;
            let stack_start = &_stack_start as *const u32;
            let mut stack_iter = stack_end;

            loop {
                stack_iter = stack_iter.offset(1);

                if stack_iter.read_volatile() != STACK_PATTERN {
                    // stack-iter points to the first used u32
                    return stack_start as usize - stack_iter as usize;
                }
            }
        }
    }
}
